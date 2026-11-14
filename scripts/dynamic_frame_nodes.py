# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Bounded DFNODE1 structural validator.

Run its adversarial probes: python3 scripts/dynamic_frame_reference.py self-check
"""
from __future__ import annotations

import re
from dynamic_frame_records import unhex


def atom(text: str) -> bytes:
    return b'' if text == '_' else unhex(text)


def node(lines: list[str], index: int = 0) -> int:
    fields = lines[index].split('\t')
    assert len(fields) == 5, 'node fields'
    tag, size_text, names, payload, children_text = fields
    assert re.fullmatch(r'0|[1-9][0-9]*', size_text) and re.fullmatch(r'0|[1-9][0-9]*', children_text)
    size, children = int(size_text), int(children_text)
    assert size <= 10000 and children <= 10000
    assert tag in ('NULL', 'missing', 'logical', 'integer', 'double', 'character', 'symbol',
                   'environment', 'closure', 'list', 'pairlist', 'language'), 'unknown type'
    if names != '-':
        assert len(names.split(',')) == size
        for name in names.split(','):
            _ = atom(name).decode('utf-8')
    if tag in ('NULL', 'missing'):
        assert size == children == 0 and payload == '_' and names == '-'
    elif tag in ('closure', 'environment', 'symbol'):
        assert size == 1 and names == '-'
        assert children == (3 if tag == 'closure' else 0)
        if tag == 'closure':
            assert payload == '_'
        elif tag == 'environment':
            assert atom(payload).decode() in {','.join(v) for v in (
                ('FALSE',)*4, ('TRUE','FALSE','FALSE','FALSE'), ('FALSE','TRUE','FALSE','FALSE'),
                ('FALSE','FALSE','TRUE','FALSE'), ('FALSE','FALSE','FALSE','TRUE'))}
        else:
            assert atom(payload)
    elif tag in ('list', 'pairlist', 'language'):
        assert children == size and payload == '_'
        assert tag != 'language' or size > 0
    else:
        assert children == 0
        values = [] if size == 0 and payload == '_' else payload.split(',')
        assert len(values) == size
        for value in values:
            if value == 'NA':
                continue
            if tag == 'character':
                _ = atom(value).decode('utf-8')
            elif tag == 'logical':
                assert value in ('0', '1')
            elif tag == 'integer':
                assert re.fullmatch(r'-?(0|[1-9][0-9]*)', value)
                assert -2147483647 <= int(value) <= 2147483647
            else:
                assert re.fullmatch(r'-?(?:0x[0-9a-f]+(?:\.[0-9a-f]+)?p[+-][0-9]+|Inf)', value)
    index += 1
    for child in range(children):
        if tag == 'closure' and child in (0, 2):
            assert lines[index].split('\t')[0] in (('NULL','pairlist') if child == 0 else ('environment',))
        index = node(lines, index)
    assert lines[index] == 'end', 'child boundary'
    return index + 1


def structure(data: bytes) -> tuple[str, str]:
    lines = data.decode().splitlines()
    assert data.endswith(b'\n') and lines[:1] == ['DFNODE1']
    assert lines[1] in ('visible', 'invisible')
    assert node(lines, 2) == len(lines), 'trailing nodes'
    return lines[1], lines[2].split('\t')[0]
