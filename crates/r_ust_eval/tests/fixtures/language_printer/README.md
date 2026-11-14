# Fixed C-profile canonical language printer

The 1539-row R 4.6.0 fixture is unchanged from the independent capture, SHA-256
`9ba1defd504a914beb7cdcd78799590b29e3628d30d9b8cc65b9c977929d7de7`.
`cases.tsv` and `generate.R` reproduce the independent R inputs. The separate
10-row `inlist-r-4.6.0.tsv` captures replacement and user-operator prefix
layout. It does not replace any original row.

The separate 45-row frozen quote fixture carries independent R-constructed trees
whose output matches the original language/symbol print sections byte for byte.
The original 69 captures, including atomic and terminal rows, remain unchanged.
The 8-row namespace fixture checks the pinned runtime branch for unusual arities
and computed operands. All three addenda use the same typed transport.
The separate 6-row computed-head fixture checks parentheses for return and
foreign calls, with ordinary and primitive function-call controls.

`outcomes.tsv` declares the SDK boundary independently of candidate output:
1401 rows require exact R bytes, 111 require MalformedCall, and 27 require an
unsupported text/encoding profile. These counts cover every original row.
Malformed calls include the source-derived rejection of unary-not with arity
other than one and assignment with arity other than two, including nested
occurrences. Incidental R rendering of such constructed calls is not an accepted
canonical SDK shape.
Within canonical language, an SDK integer payload Some(i32::MIN) is also rejected: R reserves INT_MIN for
NA_INTEGER, represented here by None. This invalid payload has no R integer
oracle row. The source boundary is R 4.6.0 arithmetic.c:143,320-321.

Node transport is hex-encoded JSON arrays. Strings carry their R Encoding tag;
the Rust test reader retains the complete fixture but rejects non-ASCII data
because Sexp has no encoding provenance. Native byte sequences and UTF8-marked
R strings can have identical bytes and different C-locale output. ASCII data
is unaffected by those tags. Tests construct Sexp directly from this independent
typed transport and do not parse R source or obtain expected bytes from Rust.

The printer uses source-free cutoff 60 bytes, backticks, integer/NA preservation,
digits 15 and scipen 0. The numeric leaf's separate immutable 5574-row fixture
remains its existing test owner. Public quote, public deparse, runtime values,
general atomic vector deparse, attributes and Unicode profiles remain outside
this checkpoint.

The source algorithm owners are R 4.6.0 `src/main/deparse.c` (traversal,
precedence and layout), `names.c` (PP metadata), `attrib.c:132` (pairlist names
derived from argument tags), `gram.y:3354` (name validity), and `printutils.c`
(literal escaping). The existing pure real-literal module retains its own source
attribution and immutable numeric tests.

The public Rust `format_sexp(&Sexp)` API now returns `Result<String, EvalError>`.
Embedding callers must propagate or handle failures. Frontend auto-print and
builtin print format a complete value before emitting its stdout event. Earlier
evaluation effects and warning/output errors retain their existing handling.
