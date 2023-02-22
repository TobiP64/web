// MIT License
//
// Copyright (c) 2019-2023 Tobias Pfeiffer
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Domain Name System
//!
//! [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035)
//! [RFC 1183](https://datatracker.ietf.org/doc/html/rfc1183)
//! [RFC 1637](https://datatracker.ietf.org/doc/html/rfc1637)
//! [RFC 1876](https://datatracker.ietf.org/doc/html/rfc1876)
//! [RFC 1996](https://datatracker.ietf.org/doc/html/rfc1996)
//! [RFC 2136](https://datatracker.ietf.org/doc/html/rfc2136)

pub mod wire;
pub mod records;
pub mod zone;
pub mod traits;
pub mod impls;

pub use self::{wire::*, records::*, zone::*, impls::*};

pub type DomainName = String;