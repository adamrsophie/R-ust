from __future__ import annotations

import json
import os
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Final, Literal

ROOT: Final = Path(__file__).resolve().parent.parent
FIXTURE: Final = ROOT / "crates/r_ust_eval/tests/fixtures/stop-r-4.6.0.tsv"
ARTIFACTS: Final = ROOT / "target/stop-foundation-reference"
PROFILE: Final = "# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"
Lane = Literal["stop", "error", "value", "limited-condition"]


@dataclass(frozen=True, slots=True)
class Spec:
    label: str
    source: str
    lane: Lane = "stop"


@dataclass(frozen=True, slots=True)
class Reference:
    identifier: int
    spec: Spec
    kind: str
    value: str
    message: str
    call: str
    warnings: str
    stdout: str


def specifications() -> tuple[Spec, ...]:
    rows = [Spec("empty", "stop()"), Spec("scalar", 'stop("boom")')]
    messages = (
        '""',
        '"two words"',
        '"100% %s"',
        '"first\\nsecond"',
        '"quote \\" slash \\\\"',
        '"caf\u00e9"',
        '"\\tindent"',
        "NULL",
        "TRUE",
        "FALSE",
        "NA",
        "NA_integer_",
        "NA_real_",
        "NaN",
        "Inf",
        "-Inf",
        "1L",
        "2.5",
        'c("a","b")',
        "c(1L,2L)",
        "c(TRUE,NA)",
        'list("a","b")',
        'list(c("a","b"),NULL)',
        'list(list("a"))',
        "as.integer(NULL)",
        "as.double(NULL)",
        "as.character(NULL)",
        'matrix(c("a","b"),nrow=1L)',
        'factor("a")',
    )
    for index, message in enumerate(messages):
        for domain in ("NULL", "NA"):
            rows.append(Spec(f"message-{index}-{domain}", f"stop({message},domain={domain})"))
    for index, flag in enumerate(
        (
            "TRUE",
            "FALSE",
            "NA",
            "1L",
            "0L",
            "2.5",
            '"TRUE"',
            '"FALSE"',
            '"bad"',
            "NULL",
            "c(FALSE,TRUE)",
            "c(NA,FALSE)",
            "list(TRUE)",
            "as.logical(NULL)",
        )
    ):
        rows.append(Spec(f"flag-{index}", f'stop("x",call.={flag})'))
    for index, domain in enumerate(
        ('"R-base"', '""', "TRUE", "FALSE", "1L", "0L", 'c("a","b")', "as.character(NULL)", 'list("R-base")')
    ):
        rows.append(Spec(f"domain-{index}", f'stop("x",domain={domain})', "stop" if index < 2 else "error"))
    rows.extend(
        (
            Spec("single-order", 'stop(call.={cat("C");FALSE},{cat("M");"x"})'),
            Spec("multiple-order", 'stop({cat("A");"a"},{cat("B");"b"},call.={cat("C");FALSE},domain={cat("D");NA})'),
            Spec("empty-domain-lazy", 'stop(domain={cat("NO");NA})'),
            Spec("single-domain-order", 'stop({cat("M");"x"},call.={cat("C");TRUE},domain={cat("D");NA})'),
            Spec("short-call-in-dots", 'stop("x",call=FALSE)'),
            Spec("short-domain-in-dots", 'stop("x",dom=NA)'),
            Spec("named-message", 'stop(message="x")'),
            Spec("duplicate-call", 'stop({cat("NO");"x"},call.=TRUE,call.=FALSE)', "error"),
            Spec("duplicate-domain", 'stop({cat("NO");"x"},domain=NULL,domain=NA)', "error"),
            Spec("empty-dot", "stop(,)", "error"),
            Spec("named-empty-dot", "stop(message=)", "error"),
            Spec("empty-call-default", 'stop("x",call.=)'),
            Spec("empty-domain-default", 'stop("x",domain=)'),
            Spec("undefined-message", "stop(missing_stop_message)", "error"),
            Spec("undefined-call-single", 'stop({cat("M");"x"},call.=missing_stop_flag)', "error"),
            Spec("undefined-call-multiple", 'stop({cat("NO");"a"},"b",call.=missing_stop_flag)', "error"),
            Spec("undefined-domain", 'stop({cat("M");"x"},domain=missing_stop_domain)', "error"),
            Spec("after-stop-absent", 'cat("BEFORE");stop("boom");cat("AFTER")'),
            Spec("saved-alias", 's<-stop;stop<-function(...)99L;s("alias")'),
            Spec("nonfunction-mask", 'stop<-1L;stop("base")'),
            Spec(
                "caller-helper-masks",
                'h<-function(){inherits<-function(...)TRUE;.makeMessage<-function(...)"wrong";.Internal<-function(...)99L;stop("right")};h()',
            ),
            Spec(
                "caller-argument-helper", 'h<-function(){as.character<-function(x)"caller";stop(as.character(1L))};h()'
            ),
            Spec("caller-domain", 'h<-function(){d<-NA;stop(c("a","b"),domain=d)};h()'),
            Spec("forwarded-unused", "g<-function()NULL;f<-function(...)g(...);f(unused=stop('forced unused'))"),
            Spec("direct-unused-lazy", 'g<-function()NULL;g(unused=stop("NO"))', "error"),
            Spec("forwarded-two", 'f<-function(...)stop(...);f("a","b")'),
            Spec("forwarded-three", 'g<-function(...)stop(...);f<-function(...)g(...);f("a","b")'),
            Spec("single-cache", 'f<-function(x){x;stop(x)};f({cat("ONCE");"x"})'),
            Spec("single-message-transfer", "(function(){stop(return(42L));99L})()", "value"),
            Spec("call-transfer", '(function(){stop("x",call.=return(42L));99L})()', "value"),
            Spec("domain-transfer", '(function(){stop("x",domain=return(42L));99L})()', "value"),
            Spec("empty-domain-transfer-lazy", "(function(){stop(domain=return(42L));99L})()"),
            Spec("break-origin", 'for(i in 1:2){stop(break);cat("NO")};42L', "value"),
            Spec("next-origin", 'for(i in 1:2){stop(next);cat("NO")};42L', "value"),
            Spec("user-override", "stop<-function(x)x+1L;stop(2L)", "value"),
            Spec("closure-value", "typeof(stop)", "value"),
            Spec("alias-value", "s<-stop;typeof(s)", "value"),
            Spec(
                "condition-object",
                'stop(structure(list(message="custom",call=NULL),class=c("custom","error","condition")))',
                "limited-condition",
            ),
        )
    )
    assert len({row.label for row in rows}) == len(rows)
    assert all(not any(c in row.source for c in "\t\r\n") for row in rows)
    return tuple(rows)


def environment() -> Mapping[str, str]:
    return dict(os.environ, R_DEFAULT_PACKAGES="NULL", R_ENABLE_JIT="0", LC_ALL="en_US.UTF-8", LANG="en_US.UTF-8")


def live_profile() -> tuple[str, ...]:
    program = (
        'cat(paste("#",R.version.string,R.version$platform,"LDOUBLE",.Machine$sizeof.longdouble,'
        '"JIT",compiler::enableJIT(-1L),"DIRECT_TOP_LEVEL"),"\\n",sep="")'
    )
    profiles: list[str] = []
    for executable in ("R", "Rscript"):
        command = (executable, "--vanilla", "--slave", "-e", program)
        result = subprocess.run(command, env=environment(), capture_output=True, timeout=30, check=False)
        assert result.returncode == 0, result.stderr.decode()
        assert result.stderr == b"", result.stderr.decode()
        profile = result.stdout.decode().rstrip("\n")
        assert profile == PROFILE, (executable, profile, PROFILE)
        profiles.append(profile)
    return tuple(profiles)


def generate_reference() -> None:
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    program = [
        "options(warn=1,warnPartialMatchArgs=FALSE)",
        "stopifnot(getRversion()=='4.6.0',R.version$arch=='aarch64',.Machine$sizeof.longdouble==8L,compiler::enableJIT(-1L)==0L)",
        "hex<-function(x){b<-charToRaw(paste(x,collapse='\\n'));if(length(b))paste(sprintf('%02x',as.integer(b)),collapse='')else'-'}",
        f".__stop_file<-file({json.dumps(str(FIXTURE))},'wb')",
        f"writeLines({json.dumps(PROFILE)},.__stop_file,useBytes=TRUE)",
    ]
    for index, spec in enumerate(specifications(), 1):
        program.extend(
            (
                ".__stop_failed<-FALSE;.__stop_error<-character();.__stop_call<-character();.__stop_warnings<-character();.__stop_result<-NULL",
                ".__stop_sink<-rawConnection(raw(),'w');sink(.__stop_sink)",
                "withCallingHandlers(tryCatch({.__stop_result<-{"
                + spec.source
                + "}},error=function(e){.__stop_failed<<-TRUE;.__stop_error<<-conditionMessage(e);.__stop_call<<-if(is.null(conditionCall(e)))character()else paste(deparse(conditionCall(e),width.cutoff=500L),collapse='\\n')}),warning=function(w){if(getOption('warn')>=2)return(invisible(NULL));if(getOption('warn')>=0).__stop_warnings<<-c(.__stop_warnings,conditionMessage(w));invokeRestart('muffleWarning')})",
                "sink();.__stop_bytes<-rawConnectionValue(.__stop_sink);close(.__stop_sink)",
                "writeLines(paste("
                + ",".join(
                    (
                        str(index),
                        json.dumps(spec.label),
                        json.dumps(spec.lane),
                        json.dumps(spec.source),
                        "if(.__stop_failed)'error'else typeof(.__stop_result)",
                        "if(.__stop_failed)'-'else hex(paste(as.character(.__stop_result),collapse='|'))",
                        "hex(.__stop_error)",
                        "hex(.__stop_call)",
                        "hex(.__stop_warnings)",
                        "if(length(.__stop_bytes))paste(sprintf('%02x',as.integer(.__stop_bytes)),collapse='')else'-'",
                    )
                )
                + ",sep='\\t'),.__stop_file,useBytes=TRUE)",
                "options(warn=1,warnPartialMatchArgs=FALSE)",
                ".__stop_remove<-setdiff(ls(all.names=TRUE),'hex');.__stop_remove<-.__stop_remove[!startsWith(.__stop_remove,'.__stop_')];if(length(.__stop_remove))rm(list=.__stop_remove,envir=.GlobalEnv)",
            )
        )
    program.append("close(.__stop_file)")
    source = ARTIFACTS / "reference.R"
    _ = source.write_text("\n".join(program) + "\n")
    result = subprocess.run(
        ("Rscript", "--vanilla", str(source)), env=environment(), capture_output=True, timeout=60, check=False
    )
    _ = (ARTIFACTS / "stdout.raw").write_bytes(result.stdout)
    _ = (ARTIFACTS / "stderr.raw").write_bytes(result.stderr)
    assert result.returncode == 0, result.stderr.decode()
    assert len(read_reference()) == len(specifications())


def read_reference() -> tuple[Reference, ...]:
    lines = FIXTURE.read_text().splitlines()
    assert lines[0] == PROFILE
    rows: list[Reference] = []
    for index, (line, spec) in enumerate(zip(lines[1:], specifications(), strict=True), 1):
        fields = line.split("\t")
        assert len(fields) == 10 and fields[:4] == [str(index), spec.label, spec.lane, spec.source]
        decoded = tuple("" if field == "-" else bytes.fromhex(field).decode() for field in fields[5:])
        rows.append(Reference(index, spec, fields[4], *decoded))
    return tuple(rows)
