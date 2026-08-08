#!/usr/bin/env python3
"""doccheck.py — worker per a scripts/doccheck.sh.

Extreu els blocs ```koto de docs/libs/*.md que contenen parelles
print!/check! (o print/check!), els transforma en scripts koto executables
i els executa amb el binari koto_calc, comparant la sortida capturada amb
les línies esperades (check!). Exit 0 si tots els exemples passen, exit 1
si algun falla.

Ús:
    doccheck.py --bin <binari> --docs <directori-docs>

Convencions dels exemples (documentades a docs/libs/*.md):
  * `print! <expr>`            — imprimeix la representació (display) de expr
  * `print! <expr1>, <expr2>`  — imprimeix la tupla (expr1, expr2)
  * `print! x = <expr>`        — assigna i imprimeix x
  * `print! x op= <expr>`      — assignació composta i imprimeix x
  * `print <expr>` (pla)       — statement koto real, es deixa tal qual
  * `check! <línia>`           — línia de sortida esperada; `check!` sol =
                                 línia buida. Diverses línies check! seguides
                                 cobreixen sortides multi-línia.
  * blocs ```koto,skip_run     — s'ignoren (exemples no executables)

Transformacions: print! no és koto real (macro de la doc), es converteix en
`print(...)` o en `doccheck_val = <expr>; print(doccheck_val)` (quan
l'expressió és un command call amb comes, que no es pot niar dins de
`print(...)`).

Normalització de la comparació: es comparen línia a línia després de
treure espais finals i les línies buides finals (el newline final d'un
print! és un artefacte, no forma part de l'exemple).
"""
import argparse
import difflib
import os
import re
import subprocess
import sys
import tempfile

# Blocs d'algebraeon.md usen els tipus i funcions d'Algebraeon com a globals
# sense import; la resta de docs s'auto-importen o usen el mòdul qualificat.
IMPORT_PREFIX = {
    "algebraeon.md": (
        "from algebraeon import N, Z, Q, Poly, Mat, Quat, Alg, "
        "Ideal, Zn, FF, CF, Perm, Group, ComplexAlg, "
        "gcd, lcm, legendre, jacobi, kronecker, eulers_constant\n"
    ),
}

# Docs VENUDES d'upstream (koto) amb exemples que fan servir sintaxi de
# versions antigues de koto (drift conegut, no arreglable aquí sense canviar
# les docs alienes): les fallades hi compten com a AVÍS, no com a error.
# Les docs pròpies (algebraeon.md) són estrictes.
KNOWN_DRIFT = {"geometry.md"}

FENCE_OPEN = re.compile(r"^```koto(.*)$")
FENCE_CLOSE = re.compile(r"^```\s*$")
PRINT_MACRO = re.compile(r"^\s*print!(?:\s|$)")
PRINT_PLAIN = re.compile(r"^\s*print(?:\s|$)")
CHECK_LINE = re.compile(r"^\s*check!(?:\s|$)")


def strip_comment(text):
    """Elimina un comentari koto final (# ...) fora de strings/claudàtors."""
    quote = None
    depth = 0
    i = 0
    while i < len(text):
        c = text[i]
        if quote:
            if c == quote and (i == 0 or text[i - 1] != "\\"):
                quote = None
            i += 1
        elif c in "'\"":
            quote = c
            i += 1
        elif c in "([{":
            depth += 1
            i += 1
        elif c in ")]}":
            depth -= 1
            i += 1
        elif c == "#" and depth == 0:
            return text[:i]
        else:
            i += 1
    return text


def split_top_level(text):
    """Divideix text en segments separats per comes de nivell superior
    (fora de strings i de () [] {})."""
    parts, depth, i, start, quote = [], 0, 0, 0, None
    while i < len(text):
        c = text[i]
        if quote:
            if c == quote and (i == 0 or text[i - 1] != "\\"):
                quote = None
            i += 1
        elif c in "'\"":
            quote = c
            i += 1
        elif c in "([{":
            depth += 1
            i += 1
        elif c in ")]}":
            depth -= 1
            i += 1
        elif c == "," and depth == 0:
            parts.append(text[start:i])
            start = i + 1
            i += 1
        else:
            i += 1
    parts.append(text[start:])
    return parts


def transform_print_line(line):
    """Converteix una línia `print! <rest>` en codi koto executable."""
    rest = line[line.index("print!") + len("print!"):].lstrip()
    rest = strip_comment(rest).strip()
    if not rest:
        return None  # print! sense expressió: no es pot verificar

    # print! x = <expr>  (no confondre amb ==)
    m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*", rest)
    if m and not rest[m.end():].startswith("="):
        name, rhs = m.group(1), rest[m.end():].strip()
        return f"{name} = {rhs}\nprint({name})"

    # print! x op= <expr>
    m = re.match(r"^([A-Za-z_][A-Za-z0-9_]*)\s*(\+=|-=|\*=|/=|%=)\s*(.+)$", rest)
    if m:
        name, op, rhs = m.group(1), m.group(2), m.group(3)
        return f"{name} {op} {rhs}\nprint({name})"

    # print! <expr> / print! <a>, <b>  (tupla) / command call amb comes
    segments = [s.strip() for s in split_top_level(rest)]
    if len(segments) == 1 and not re.search(r"\s", rest):
        return f"print({rest})"
    if len(segments) > 1 and all(not re.search(r"\s", s) for s in segments):
        return f"print({rest})"
    # command call (o expressió amb espais): cal una variable temporal,
    # `print(...)` no admet command calls niats com a arguments
    return f"doccheck_val = {rest}\nprint(doccheck_val)"


def transform_block(lines):
    """Converteix un bloc koto de la doc en (codi koto, línies esperades)."""
    code, expected = [], []
    has_print, has_check = False, False
    for line in lines:
        if CHECK_LINE.match(line):
            has_check = True
            content = line[line.index("check!") + len("check!"):]
            if content.startswith(" "):
                content = content[1:]
            expected.append(content.rstrip())
        elif PRINT_MACRO.match(line):
            has_print = True
            transformed = transform_print_line(line)
            if transformed is None:
                return None, None, None
            code.append(transformed)
        else:
            code.append(line)
            if PRINT_PLAIN.match(line):
                has_print = True
    return "\n".join(code) + "\n", expected, (has_print, has_check)


def extract_blocks(md_path):
    """Extreu els blocs ```koto (o ```koto,skip_run) d'un fitxer markdown."""
    blocks = []
    with open(md_path, encoding="utf-8") as f:
        in_block, skip, buf = False, False, []
        for raw in f:
            line = raw.rstrip("\n")
            if not in_block:
                m = FENCE_OPEN.match(line)
                if m:
                    in_block = True
                    attrs = m.group(1).strip()
                    skip = "skip_run" in attrs
                    buf = []
            else:
                if FENCE_CLOSE.match(line):
                    if not skip:
                        blocks.append(buf)
                    in_block = False
                else:
                    buf.append(line)
    return blocks


def run_block(bin_path, code):
    fd, path = tempfile.mkstemp(suffix=".koto", prefix="doccheck_")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(code)
        return subprocess.run([bin_path, path], capture_output=True, text=True)
    finally:
        try:
            os.unlink(path)
        except OSError:
            pass


def check_block(bin_path, block_lines, doc_name, block_no, prefix=""):
    code, expected, flags = transform_block(block_lines)
    if code is None or flags is None:
        return None  # no verificable
    code = prefix + code
    has_print, has_check = flags
    if not has_print and not has_check:
        return None  # bloc sense print/check: no és un exemple verificable
    if not has_print:
        return "SKIP-check-without-print"
    if not has_check:
        return "SKIP-print-without-check"

    proc = run_block(bin_path, code)
    actual = proc.stdout.splitlines()
    while expected and expected[-1] == "":
        expected.pop()
    while actual and actual[-1] == "":
        actual.pop()

    if proc.returncode != 0:
        return ("FAIL", code, expected, actual,
                "koto error (exit %d): %s" % (proc.returncode,
                                              proc.stderr.strip().splitlines()[-1]
                                              if proc.stderr.strip() else "?"))
    if actual != expected:
        diff = "\n".join(difflib.ndiff(expected, actual))
        return ("FAIL", code, expected, actual,
                "sortida inesperada:\n%s" % diff)
    return ("OK", code, expected, actual, None)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True)
    ap.add_argument("--docs", required=True)
    args = ap.parse_args()

    md_files = sorted(
        f for f in os.listdir(args.docs)
        if f.endswith(".md") and os.path.isfile(os.path.join(args.docs, f))
    )
    if not md_files:
        print("doccheck: no docs/libs/*.md trobats", file=sys.stderr)
        return 1

    total_blocks = total_checks = 0
    failures = []
    for md_name in md_files:
        md_path = os.path.join(args.docs, md_name)
        blocks = extract_blocks(md_path)
        prefix = IMPORT_PREFIX.get(md_name, "")
        doc_blocks = doc_checks = 0
        doc_failures = []
        for i, block_lines in enumerate(blocks, 1):
            result = check_block(args.bin, block_lines, md_name, i, prefix)
            if result is None:
                continue
            status, code, expected, actual, detail = result
            if status == "SKIP-check-without-print":
                print(f"  avís {md_name} bloc {i}: check! sense print! — s'omet", file=sys.stderr)
                continue
            if status == "SKIP-print-without-check":
                print(f"  avís {md_name} bloc {i}: print!/print sense check! — s'omet", file=sys.stderr)
                continue
            doc_blocks += 1
            doc_checks += len(expected)
            if status != "OK":
                doc_failures.append((i, detail, expected, actual))
        total_blocks += doc_blocks
        total_checks += doc_checks
        if doc_failures:
            if md_name in KNOWN_DRIFT:
                print(f"== {md_name}  AVÍS (doc d'upstream amb drift conegut)")
                for block_no, detail, expected, actual in doc_failures:
                    print(f"   avís bloc {block_no}: {detail}")
                continue
            print(f"== {md_name}  FAIL")
            for block_no, detail, expected, actual in doc_failures:
                print(f"   bloc {block_no}: {detail}")
            failures.append((md_name, doc_failures))
        else:
            print(f"== {md_name}  OK ({doc_blocks} blocs, {doc_checks} checks)")

    if failures:
        print(f"\ndoccheck: FAILED — {sum(len(f[1]) for f in failures)} "
              f"exemple(s) fallat(s) de {total_blocks} blocs ({total_checks} checks)")
        return 1
    print(f"\ndoccheck: OK — {total_blocks} blocs, {total_checks} checks, 0 fallos")
    return 0


if __name__ == "__main__":
    sys.exit(main())
