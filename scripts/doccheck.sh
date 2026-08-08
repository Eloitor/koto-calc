#!/usr/bin/env bash
# ============================================================
# doccheck.sh — valida els exemples print!/check! de les docs de koto-calc
#
# Extreu els blocs ```koto de docs/libs/*.md que contenen parelles
# print!/check! i els executa com a suite amb el binari koto_calc:
# cada `print! <expr>` s'executa i la sortida es compara amb les línies
# `check!` esperades. Exit 0 si tots els exemples passen, exit != 0 si
# algun falla (evita el drift entre les docs i la implementació).
#
# Ús:
#   ./scripts/doccheck.sh            # valida totes les docs/libs/*.md
#   ./scripts/doccheck.sh -v         # idem (verbose; mateix comportament)
#
# No toca libs/ ni docs/: només llegeix les docs i executa el binari.
# Blocs ```koto,skip_run s'ignoren (exemples no executables, p.e. paths
# de tempfile). Els blocs amb print!/check! que no es poden verificar
# s'avisen i s'ometen amb tolerància.
#
# Dependències: python3, cargo (compila el binari si cal).
# Vegeu el cap de doccheck.py per les convencions de transformació.
# ============================================================
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${HERE}/.." && pwd)"

cd "${REPO_ROOT}"

# Compila el binari si no existeix (idempotent)
BIN="${REPO_ROOT}/target/debug/koto_calc"
if [ ! -x "${BIN}" ]; then
  cargo build --quiet
fi

exec python3 "${HERE}/doccheck.py" --bin "${BIN}" --docs "${REPO_ROOT}/docs/libs"
