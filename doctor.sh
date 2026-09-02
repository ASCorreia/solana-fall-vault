#!/usr/bin/env bash
# solana-doctor — checks the toolchain for Solana Fall School

for t in rustc cargo solana anchor avm node yarn; do
  if command -v "$t" >/dev/null 2>&1; then
    printf 'ok   %-7s %s\n' "$t" "$("$t" --version 2>/dev/null | head -1)"
  else
    printf 'MISS %-7s not on PATH\n' "$t"
  fi
done

echo
solana config get 2>/dev/null | grep -E 'RPC URL|Keypair Path'
echo "address: $(solana address 2>/dev/null || echo 'no keypair yet')"
echo "balance: $(solana balance 2>/dev/null || echo 'unreachable')"
