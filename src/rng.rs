use std::ops::{Range, RangeInclusive};

pub struct RNG {
  state: u64,
}

// Generates `next_<type>` (any value of the type) and `range_<type>`, which
// takes an inclusive range such as `rng.range_i32(1..=6)`. `$u` is the
// unsigned type of the same width, used so the span max - min never overflows.
macro_rules! int_fns {
  ($next:ident, $range:ident, $t:ty, $u:ty) => {
    pub fn $next(&mut self) -> $t {
      self.next() as $t
    }

    pub fn $range(&mut self, range: RangeInclusive<$t>) -> $t {
      let (min, max) = range.into_inner();
      debug_assert!(max >= min);
      let span = max.wrapping_sub(min) as $u as u64;
      min.wrapping_add(self.below_inclusive(span) as $u as $t)
    }
  };
}

impl RNG {
  pub fn new(seed: u64) -> Self {
    RNG {
      state: if seed == 0 { 1 } else { seed },
    }
  }

  pub fn next(&mut self) -> u64 {
    self.state ^= self.state << 13;
    self.state ^= self.state >> 7;
    self.state ^= self.state << 17;
    self.state
  }

  // Returns a value in 0..=span, including the full-range case.
  fn below_inclusive(&mut self, span: u64) -> u64 {
    let x = self.next();
    if span == u64::MAX {
      x
    } else {
      x % (span + 1)
    }
  }

  pub fn next_bool(&mut self) -> bool {
    self.next() & 1 == 1
  }

  int_fns!(next_i8, range_i8, i8, u8);
  int_fns!(next_i16, range_i16, i16, u16);
  int_fns!(next_i32, range_i32, i32, u32);
  int_fns!(next_i64, range_i64, i64, u64);

  int_fns!(next_u8, range_u8, u8, u8);
  int_fns!(next_u16, range_u16, u16, u16);
  int_fns!(next_u32, range_u32, u32, u32);
  int_fns!(next_u64, range_u64, u64, u64);

  // Returns a value in [0, 1), built from the top 24 bits (the f32 mantissa
  // width) so every result is exactly representable.
  pub fn next_f32(&mut self) -> f32 {
    (self.next() >> 40) as f32 / (1u32 << 24) as f32
  }

  // Returns a value in [0, 1), built from the top 53 bits.
  pub fn next_f64(&mut self) -> f64 {
    (self.next() >> 11) as f64 / (1u64 << 53) as f64
  }

  // Returns a value in the half-open range, e.g. `rng.range_f32(0.0..1.5)`.
  pub fn range_f32(&mut self, range: Range<f32>) -> f32 {
    debug_assert!(range.start < range.end);
    range.start + self.next_f32() * (range.end - range.start)
  }

  // Returns a value in the half-open range, e.g. `rng.range_f64(0.0..1.5)`.
  pub fn range_f64(&mut self, range: Range<f64>) -> f64 {
    debug_assert!(range.start < range.end);
    range.start + self.next_f64() * (range.end - range.start)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const DRAWS: usize = 100_000;

  #[test]
  fn same_seed_gives_same_sequence() {
    let mut a = RNG::new(42);
    let mut b = RNG::new(42);
    for _ in 0..100 {
      assert_eq!(a.next(), b.next());
    }
  }

  #[test]
  fn zero_seed_does_not_get_stuck() {
    // xorshift maps 0 to 0 forever, so new() replaces a 0 seed with 1.
    let mut r = RNG::new(0);
    assert_ne!(r.next(), 0);
  }

  #[test]
  fn int_range_stays_in_bounds() {
    let mut r = RNG::new(42);
    for _ in 0..DRAWS {
      assert!((-3..=5).contains(&r.range_i32(-3..=5)));
      assert!((10..=20).contains(&r.range_u16(10..=20)));
    }
  }

  #[test]
  fn int_range_hits_both_ends() {
    let mut r = RNG::new(42);
    let (mut lo, mut hi) = (i8::MAX, i8::MIN);
    for _ in 0..DRAWS {
      let v = r.range_i8(i8::MIN..=i8::MAX);
      lo = lo.min(v);
      hi = hi.max(v);
    }
    assert_eq!((lo, hi), (i8::MIN, i8::MAX));
  }

  #[test]
  fn int_range_single_value() {
    let mut r = RNG::new(42);
    assert_eq!(r.range_u8(7..=7), 7);
    assert_eq!(r.range_i64(-9..=-9), -9);
  }

  #[test]
  fn full_64_bit_ranges_do_not_overflow() {
    let mut r = RNG::new(42);
    for _ in 0..1000 {
      r.range_i64(i64::MIN..=i64::MAX);
      r.range_u64(0..=u64::MAX);
    }
  }

  #[test]
  fn unit_floats_are_half_open() {
    let mut r = RNG::new(42);
    for _ in 0..DRAWS {
      let f = r.next_f32();
      assert!((0.0..1.0).contains(&f));
      let d = r.next_f64();
      assert!((0.0..1.0).contains(&d));
    }
  }

  #[test]
  fn float_range_stays_in_bounds() {
    let mut r = RNG::new(42);
    for _ in 0..DRAWS {
      assert!((-1.0..2.0).contains(&r.range_f32(-1.0..2.0)));
      assert!((-1.0..2.0).contains(&r.range_f64(-1.0..2.0)));
    }
  }

  #[test]
  #[should_panic]
  #[cfg(debug_assertions)]
  fn reversed_int_range_panics_in_debug() {
    let mut r = RNG::new(42);
    #[allow(clippy::reversed_empty_ranges)]
    r.range_i32(5..=1);
  }
}
