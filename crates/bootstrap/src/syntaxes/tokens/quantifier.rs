/*     _              _ _
 *  __| |_ _ ___ _ __( |_)_ _
 * / _` | '_/ _ \ '_ \/| | ' \
 * \__,_|_| \___/ .__/ |_|_||_| dropin-compiler - WebAssembly
 *              |_|
 * Copyright © 2019-2022 Blue Forest
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::iter::Peekable;
use std::str::CharIndices;

use crate::syntaxes::{Expression, ParseError, Patterns};
use super::Token;

#[derive(Debug)]
pub struct Quantifier<'a> {
  ranges: Vec<(Option<u32>, Option<u32>)>,
  token:  Box<dyn Token<'a> + 'a>,
}

impl<'a> Quantifier<'a> {
  pub fn detect(c: char) -> bool {
    c == '{'
  }

  pub fn new(
    syntax: &'a str,
    iter:   &mut Peekable<CharIndices<'a>>,
    token:  Box<dyn Token<'a> + 'a>
  ) -> Self {
    iter.next(); // skip '{'
    let ranges = Parser::default().parse(syntax, iter);
    Self{ ranges, token }
  }
}

struct Parser {
  is_parsing_min: bool,
  current_min: Option<u32>,
  quantity_start: Option<usize>,
}

impl Default for Parser {
  fn default() -> Self {
    Self{
      is_parsing_min: true,
      current_min: None,
      quantity_start: None,
    }
  }
}

impl Parser {
  fn parse<'a>(
    mut self,
    syntax: &'a str,
    iter:   &mut Peekable<CharIndices<'a>>,
  ) -> Vec<(Option<u32>, Option<u32>)> {
    let mut result = Vec::new();
    loop {
      let (i, c) = iter.next().unwrap();
      match c {
        '}' => {
          result.push(self.create_range(syntax, i));
          break;
        }
        '|' => {
          result.push(self.create_range(syntax, i));
          self.is_parsing_min = true;
          self.quantity_start = None;
        }
        '.' => {
          if !self.is_parsing_min {
            panic!("unexpected '.' (max is already set)");
          }
          self.is_parsing_min = false;
          if let Some(start) = self.quantity_start {
            self.current_min = Some(Self::get_quantity(syntax, start, i));
            self.quantity_start = None;
          }
          let (_, next) = iter.next().unwrap();
          if next != '.' {
            panic!("unexpected token: {}", next);
          }
        }
        _ => {
          if !c.is_ascii_digit() {
            panic!("unexpected token: {}", c);
          }
          if self.quantity_start.is_none() {
            self.quantity_start = Some(i);
          }
        }
      }
    }
    result
  }

  fn create_range(
    &mut self,
    syntax: &str,
    i:      usize,
  ) -> (Option<u32>, Option<u32>) {
    if self.is_parsing_min {
      if let Some(start) = self.quantity_start {
        let quantity = Self::get_quantity(syntax, start, i);
        (Some(quantity), Some(quantity))
      } else {
        panic!("unexpected '|'");
      }
    } else {
      let max = if let Some(start) = self.quantity_start {
        Some(Self::get_quantity(syntax, start, i))
      } else { None };
      (self.current_min, max)
    }
  }

  #[inline(always)]
  fn get_quantity(syntax: &str, start: usize, end: usize) -> u32 {
    syntax.get(start..end).unwrap().parse::<u32>().unwrap()
  }
}

impl<'a> Token<'a> for Quantifier<'a> {
  fn parse<'b, 'c>(
    &self,
    patterns: &'c Patterns<'a>,
    module:   &'b str,
    iter:     &mut Peekable<CharIndices<'b>>,
    expr:     &mut Expression<'a, 'b>,
  ) -> Result<(), ParseError> {
    let mut n = 0;
    let err = loop {
      if let Err(err) = self.token.parse(patterns, module, iter, expr) {
        break Err(err);
      }
      n += 1;
      if let None = iter.peek() {
        break Ok(())
      }
      let (i, _) = iter.peek().unwrap();
    };
    let mut ok = false;
    for (min, max) in self.ranges.iter() {
      if let Some(min) = min {
        if *min > n {
          break;
        }
      }
      if let Some(max) = max {
        if *max > n {
          ok = true;
          break;
        }
      } else {
        ok = true;
        break;
      }
    }
    if !ok {
      panic!(
        "expected {:?} {:?}, recognized {} times",
        self.ranges, self.token, n,
      );
    } else { Ok(()) }
  }
}
