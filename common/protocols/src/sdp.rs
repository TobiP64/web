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

//! Session Description Protocol (SDP)
//!
//! [RFC 8866](https://datatracker.ietf.org/doc/html/rfc8866)

use {
	std::{ops, str::FromStr, time::Duration, fmt::{Display, Formatter, Result as FmtResult}},
	chrono::{DateTime, Utc},
	crate::utils::Url
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Description(Vec<Field>);

impl FromStr for Description {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.lines()
			.map(|l| l.parse())
			.collect::<Result<Vec<_>, _>>()
			.map(Self)
	}
}

impl Display for Description {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		if self.is_empty() {
			return Ok(());
		}
		
		write!(f, "{}", &self[0])?;
		
		for field in &self.0[1..] {
			write!(f, "\n{}", field)?;
		}
		
		Ok(())
	}
}

impl ops::Deref for Description {
	type Target = Vec<Field>;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ops::DerefMut for Description {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Field {
	ProtocolVersion(usize),
	Origin(Origin),
	SessionName(String),
	SessionInformation(String),
	Uri(Url),
	EmailAddress(String),
	PhoneNumber(String),
	ConnectionInformation(ConnectionInformation),
	BandwidthInformation(BandwidthInformation),
	TimeActive(TimeActive),
	RepeatTimes(RepeatTimes),
	TimeZoneAdjustment(TimeZoneAdjustment),
	Attribute(Attribute),
	Custom(Box<str>, Box<str>)
}

impl Field {
	pub fn is_protocol_version(&self) -> bool {
		matches!(self, Self::ProtocolVersion(_))
	}
	
	pub fn as_protocol_version(&self) -> Option<&usize> {
		match self {
			Self::ProtocolVersion(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_protocol_version(&mut self) -> Option<&mut usize> {
		match self {
			Self::ProtocolVersion(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_protocol_version(self) -> Option<usize> {
		match self {
			Self::ProtocolVersion(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_origin(&self) -> bool {
		matches!(self, Self::Origin(_))
	}
	
	pub fn as_origin(&self) -> Option<&Origin> {
		match self {
			Self::Origin(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_origin(&mut self) -> Option<&mut Origin> {
		match self {
			Self::Origin(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_origin(self) -> Option<Origin> {
		match self {
			Self::Origin(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_session_name(&self) -> bool {
		matches!(self, Self::SessionName(_))
	}
	
	pub fn as_session_name(&self) -> Option<&String> {
		match self {
			Self::SessionName(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_session_name(&mut self) -> Option<&mut String> {
		match self {
			Self::SessionName(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_session_name(self) -> Option<String> {
		match self {
			Self::SessionName(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_session_information(&self) -> bool {
		matches!(self, Self::SessionInformation(_))
	}
	
	pub fn as_session_information(&self) -> Option<&String> {
		match self {
			Self::SessionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_session_information(&mut self) -> Option<&mut String> {
		match self {
			Self::SessionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_session_information(self) -> Option<String> {
		match self {
			Self::SessionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_uri(&self) -> bool {
		matches!(self, Self::Uri(_))
	}
	
	pub fn as_uri(&self) -> Option<&Url> {
		match self {
			Self::Uri(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_uri(&mut self) -> Option<&mut Url> {
		match self {
			Self::Uri(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_uri(self) -> Option<Url> {
		match self {
			Self::Uri(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_email_address(&self) -> bool {
		matches!(self, Self::EmailAddress(_))
	}
	
	pub fn as_email_address(&self) -> Option<&String> {
		match self {
			Self::EmailAddress(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_email_address(&mut self) -> Option<&mut String> {
		match self {
			Self::EmailAddress(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_email_address(self) -> Option<String> {
		match self {
			Self::EmailAddress(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_phone_number(&self) -> bool {
		matches!(self, Self::PhoneNumber(_))
	}
	
	pub fn as_phone_number(&self) -> Option<&String> {
		match self {
			Self::PhoneNumber(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_phone_number(&mut self) -> Option<&mut String> {
		match self {
			Self::PhoneNumber(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_phone_number(self) -> Option<String> {
		match self {
			Self::PhoneNumber(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_connection_information(&self) -> bool {
		matches!(self, Self::ConnectionInformation(_))
	}
	
	pub fn as_connection_information(&self) -> Option<&ConnectionInformation> {
		match self {
			Self::ConnectionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_connection_information(&mut self) -> Option<&mut ConnectionInformation> {
		match self {
			Self::ConnectionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_connection_information(self) -> Option<ConnectionInformation> {
		match self {
			Self::ConnectionInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_bandwidth_information(&self) -> bool {
		matches!(self, Self::BandwidthInformation(_))
	}
	
	pub fn as_bandwidth_information(&self) -> Option<&BandwidthInformation> {
		match self {
			Self::BandwidthInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_bandwidth_information(&mut self) -> Option<&mut BandwidthInformation> {
		match self {
			Self::BandwidthInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_bandwidth_information(self) -> Option<BandwidthInformation> {
		match self {
			Self::BandwidthInformation(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_time_active(&self) -> bool {
		matches!(self, Self::TimeActive(_))
	}
	
	pub fn as_time_active(&self) -> Option<&TimeActive> {
		match self {
			Self::TimeActive(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_time_active(&mut self) -> Option<&mut TimeActive> {
		match self {
			Self::TimeActive(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_time_active(self) -> Option<TimeActive> {
		match self {
			Self::TimeActive(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_repeat_times(&self) -> bool {
		matches!(self, Self::RepeatTimes(_))
	}
	
	pub fn as_repeat_times(&self) -> Option<&RepeatTimes> {
		match self {
			Self::RepeatTimes(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_repeat_times(&mut self) -> Option<&mut RepeatTimes> {
		match self {
			Self::RepeatTimes(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_repeat_times(self) -> Option<RepeatTimes> {
		match self {
			Self::RepeatTimes(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_time_zone_adjustment(&self) -> bool {
		matches!(self, Self::TimeZoneAdjustment(_))
	}
	
	pub fn as_time_zone_adjustment(&self) -> Option<&TimeZoneAdjustment> {
		match self {
			Self::TimeZoneAdjustment(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_time_zone_adjustment(&mut self) -> Option<&mut TimeZoneAdjustment> {
		match self {
			Self::TimeZoneAdjustment(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_time_zone_adjustment(self) -> Option<TimeZoneAdjustment> {
		match self {
			Self::TimeZoneAdjustment(v) => Some(v),
			_ => None
		}
	}
	
	pub fn is_attribute(&self) -> bool {
		matches!(self, Self::Attribute(_))
	}
	
	pub fn as_attribute(&self) -> Option<&Attribute> {
		match self {
			Self::Attribute(v) => Some(v),
			_ => None
		}
	}
	
	pub fn as_mut_attribute(&mut self) -> Option<&mut Attribute> {
		match self {
			Self::Attribute(v) => Some(v),
			_ => None
		}
	}
	
	pub fn into_attribute(self) -> Option<Attribute> {
		match self {
			Self::Attribute(v) => Some(v),
			_ => None
		}
	}
}

impl FromStr for Field {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (ty, val) = s.split_once('=').ok_or(())?;
		
		Ok(match ty {
			"v" => Self::ProtocolVersion(val.parse().map_err(|_| ())?),
			"o" => Self::Origin(val.parse()?),
			"s" => Self::SessionName(val.to_string()),
			"i" => Self::SessionInformation(val.to_string()),
			"u" => Self::Uri(val.to_string()),
			"e" => Self::EmailAddress(val.to_string()),
			"p" => Self::PhoneNumber(val.to_string()),
			"c" => Self::ConnectionInformation(val.parse().map_err(|_| ())?),
			"b" => Self::BandwidthInformation(val.parse().map_err(|_| ())?),
			"t" => Self::TimeActive(val.parse().map_err(|_| ())?),
			"r" => Self::RepeatTimes(val.parse().map_err(|_| ())?),
			"z" => Self::TimeZoneAdjustment(val.parse().map_err(|_| ())?),
			"a" => Self::Attribute(val.parse().map_err(|_| ())?),
			_   => Self::Custom(ty.to_string().into_boxed_str(), val.to_string().into_boxed_str())
		})
	}
}

impl Display for Field {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Self::ProtocolVersion(v)       => write!(f, "v={}", v),
			Self::Origin(v)                => write!(f, "o={}", v),
			Self::SessionName(v)           => write!(f, "s={}", v),
			Self::SessionInformation(v)    => write!(f, "i={}", v),
			Self::Uri(v)                   => write!(f, "u={}", v),
			Self::EmailAddress(v)          => write!(f, "e={}", v),
			Self::PhoneNumber(v)           => write!(f, "p={}", v),
			Self::ConnectionInformation(v) => write!(f, "c={}", v),
			Self::BandwidthInformation(v)  => write!(f, "b={}", v),
			Self::TimeActive(v)            => write!(f, "t={}", v),
			Self::RepeatTimes(v)           => write!(f, "r={}", v),
			Self::TimeZoneAdjustment(v)    => write!(f, "z={}", v),
			Self::Attribute(v)             => write!(f, "a={}", v),
			Self::Custom(name, value)      => write!(f, "{}={}", name, value),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Origin {
	pub username:        String,
	pub session_id:      u64,
	pub session_version: u64,
	pub net_type:        NetType,
	pub addr_type:       AddrType,
	pub unicast_addr:    std::net::IpAddr,
}

impl FromStr for Origin {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut values = s.split_ascii_whitespace();
		Ok(Self {
			username:        values.next().ok_or(())?.to_string(),
			session_id:      values.next().ok_or(())?.parse().map_err(|_| ())?,
			session_version: values.next().ok_or(())?.parse().map_err(|_| ())?,
			net_type:        values.next().ok_or(())?.parse()?,
			addr_type:       values.next().ok_or(())?.parse()?,
			unicast_addr:    values.next().ok_or(())?.parse().map_err(|_| ())?
		})
	}
}

impl Display for Origin {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(
			f,
			"{} {} {} {} {} {}",
			&self.username,
			self.session_id,
			self.session_version,
			self.net_type,
			self.addr_type,
			self.unicast_addr
		)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NetType {
	IP4,
	IP6
}

impl FromStr for NetType {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"IP4" => Self::IP4,
			"IP6" => Self::IP6,
			_     => return Err(())
		})
	}
}

impl Display for NetType {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::IP4 => "IP4",
			Self::IP6 => "IP6"
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AddrType {
	IN
}

impl FromStr for AddrType {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"IN" => Self::IN,
			_    => return Err(())
		})
	}
}

impl Display for AddrType {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::IN => "IN",
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ConnectionInformation {
	pub net_type:        NetType,
	pub addr_type:       AddrType,
	pub connection_addr: std::net::IpAddr
}

impl FromStr for ConnectionInformation {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut values = s.split_ascii_whitespace();
		Ok(Self {
			net_type:        values.next().ok_or(())?.parse().map_err(|_| ())?,
			addr_type:       values.next().ok_or(())?.parse().map_err(|_| ())?,
			connection_addr: values.next().ok_or(())?.parse().map_err(|_| ())?
		})
	}
}

impl Display for ConnectionInformation {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{} {} {}", self.net_type, self.addr_type, self.connection_addr)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BandwidthInformation {
	pub r#type:    BandwidthType,
	pub bandwidth: String
}

impl FromStr for BandwidthInformation {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (ty, bw) = s.split_once(':').ok_or(())?;
		Ok(Self {
			r#type:    ty.parse().map_err(|_| ())?,
			bandwidth: bw.to_string()
		})
	}
}

impl Display for BandwidthInformation {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{}:{}", self.r#type, &self.bandwidth)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BandwidthType {
	CT,
	AS
}

impl FromStr for BandwidthType {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"CT" => Self::CT,
			"AS" => Self::AS,
			_    => return Err(())
		})
	}
}

impl Display for BandwidthType {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		f.write_str(match self {
			Self::CT => "CT",
			Self::AS => "AS"
		})
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeActive {
	pub start: DateTime<Utc>,
	pub stop:  DateTime<Utc>
}

impl FromStr for TimeActive {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (start, stop) = s.split_once(' ').ok_or(())?;
		Ok(Self {
			start: crate::utils::ntp_to_date_time(start.parse().map_err(|_| ())?),
			stop:  crate::utils::ntp_to_date_time(stop.parse().map_err(|_| ())?),
		})
	}
}

impl Display for TimeActive {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{} {}", crate::utils::date_time_to_ntp(self.start),
			   crate::utils::date_time_to_ntp(self.stop))
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepeatTimes {
	pub interval: Duration,
	pub duration: Duration,
	pub offsets:  Vec<Duration>
}

impl FromStr for RepeatTimes {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut values = s.split_ascii_whitespace();
		Ok(Self {
			interval: Duration::from_secs(values.next().ok_or(())?.parse().map_err(|_| ())?),
			duration: Duration::from_secs(values.next().ok_or(())?.parse().map_err(|_| ())?),
			offsets:  values
				.map(|v| v.parse().map(Duration::from_secs))
				.collect::<Result<Vec<_>, _>>()
				.map_err(|_| ())?
		})
	}
}

impl Display for RepeatTimes {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "{} {}", self.interval.as_secs(), self.duration.as_secs())?;
		
		for offset in &self.offsets {
			write!(f, " {}", offset.as_secs())?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeZoneAdjustment {

}

impl FromStr for TimeZoneAdjustment {
	type Err = ();
	
	fn from_str(_s: &str) -> Result<Self, Self::Err> {
		Ok(Self {})
	}
}

impl Display for TimeZoneAdjustment {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		write!(f, "")
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
	pub name:  String,
	pub value: String
}

impl FromStr for Attribute {
	type Err = ();
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (name, value) = s.split_once(':').ok_or(())?;
		Ok(Self {
			name:  name.to_string(),
			value: value.to_string()
		})
	}
}

impl Display for Attribute {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(f, "{}:{}", &self.name, &self.value)
	}
}