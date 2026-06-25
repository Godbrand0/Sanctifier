import { analyzeSorobanSource, looksLikeSorobanSource, CODES } from './analyzer';

describe('looksLikeSorobanSource', () => {
  it('returns true for soroban_sdk import', () => {
    expect(looksLikeSorobanSource('use soroban_sdk::Env;')).toBe(true);
  });

  it('returns true for #[contractimpl]', () => {
    expect(looksLikeSorobanSource('#[contractimpl]\nimpl MyContract {}')).toBe(true);
  });

  it('returns false for plain Rust', () => {
    expect(looksLikeSorobanSource('fn main() { println!("hello"); }')).toBe(false);
  });
});

describe('analyzeSorobanSource — panic detection', () => {
  const src = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn risky(_env: Env) {
    panic!("not allowed");
  }
}
`;

  it('detects panic! as S002 error', () => {
    const findings = analyzeSorobanSource(src);
    const panics = findings.filter((f) => f.code === CODES.PANIC_USAGE);
    expect(panics.length).toBeGreaterThan(0);
    expect(panics[0].severity).toBe('error');
  });
});

describe('analyzeSorobanSource — unwrap detection', () => {
  const src = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn risky(env: Env) {
    let val = env.storage().persistent().get(&0u32).unwrap();
    let _ = val;
  }
}
`;

  it('detects .unwrap() as S006 warning', () => {
    const findings = analyzeSorobanSource(src);
    const unsafe = findings.filter((f) => f.code === CODES.UNSAFE_PATTERN);
    expect(unsafe.length).toBeGreaterThan(0);
    expect(unsafe[0].severity).toBe('warning');
  });
});

describe('analyzeSorobanSource — auth gap detection', () => {
  const src = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn store(env: Env, key: u32, val: u32) {
    env.storage().persistent().set(&key, &val);
  }
}
`;

  it('flags missing require_auth as S001', () => {
    const findings = analyzeSorobanSource(src);
    const authGaps = findings.filter((f) => f.code === CODES.AUTH_GAP);
    expect(authGaps.length).toBeGreaterThan(0);
  });

  it('does NOT flag when require_auth is present', () => {
    const safeSrc = `
use soroban_sdk::{Env, Address};
#[contractimpl]
impl MyContract {
  pub fn store(env: Env, caller: Address, key: u32, val: u32) {
    caller.require_auth();
    env.storage().persistent().set(&key, &val);
  }
}
`;
    const findings = analyzeSorobanSource(safeSrc);
    const authGaps = findings.filter((f) => f.code === CODES.AUTH_GAP);
    expect(authGaps).toHaveLength(0);
  });
});

describe('analyzeSorobanSource — arithmetic heuristic', () => {
  const src = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn add(env: Env, a: i128, b: i128) -> i128 {
    let _ = env;
    a + b
  }
}
`;

  it('flags unchecked arithmetic as S003', () => {
    const findings = analyzeSorobanSource(src);
    const overflow = findings.filter((f) => f.code === CODES.ARITHMETIC_OVERFLOW);
    expect(overflow.length).toBeGreaterThan(0);
  });

  it('does NOT flag checked_add', () => {
    const safeSrc = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn add(env: Env, a: i128, b: i128) -> i128 {
    let _ = env;
    a.checked_add(b).unwrap_or(i128::MAX)
  }
}
`;
    const findings = analyzeSorobanSource(safeSrc);
    const overflow = findings.filter((f) => f.code === CODES.ARITHMETIC_OVERFLOW);
    expect(overflow).toHaveLength(0);
  });
});

describe('analyzeSorobanSource — de-duplication', () => {
  it('does not return duplicate findings for the same line/code', () => {
    const src = `
use soroban_sdk::Env;
#[contractimpl]
impl MyContract {
  pub fn risky(_env: Env) { panic!("a"); }
}
`;
    const findings = analyzeSorobanSource(src);
    const panics = findings.filter((f) => f.code === CODES.PANIC_USAGE);
    const lines = panics.map((f) => f.line);
    const unique = [...new Set(lines)];
    expect(lines.length).toBe(unique.length);
  });
});
