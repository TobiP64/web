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

use {
	crate::dns::DomainName,
	std::{io, str::FromStr, fmt}
};
use crate::dns::{ZoneParseError, ZoneParseErrorType};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRecord {
	pub name:   String,
	pub ttl:    u32,
	pub class:  Class,
	pub data:   ResourceRecordData
}

impl FromStr for ResourceRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for ResourceRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {} {} {}", &self.name, self.ttl, self.class, &self.data)
	}
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Class {
	IN   = 1,
	CS   = 2,
	CH   = 3,
	HS   = 4,
	None = 254,
	Any  = 255
}

impl TryFrom<u16> for Class {
	type Error = ();
	
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1..=4 | 245..=255 => Ok(unsafe { std::mem::transmute(value) }),
			_ => Err(())
		}
	}
}

impl FromStr for Class {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"IN" => Self::IN,
			"CS" => Self::CS,
			"CH" => Self::CH,
			"HS" => Self::HS,
			"*"  => Self::Any,
			_    => return Err(ZoneParseError::new(ZoneParseErrorType::RecordClass, 0, 0))
		})
	}
}

impl fmt::Display for Class {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::IN   => "IN",
			Self::CS   => "CS",
			Self::CH   => "CH",
			Self::HS   => "HS",
			Self::None => return Err(fmt::Error),
			Self::Any  => "*",
		})
	}
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Type {
	A          = 1,
	NS         = 2,
	MD         = 3,
	MF         = 4,
	CName      = 5,
	SOA        = 6,
	MB         = 7,
	MG         = 8,
	MR         = 9,
	Null       = 10,
	WKS        = 11,
	Ptr        = 12,
	HInfo      = 13,
	MInfo      = 14,
	MX         = 15,
	TXT        = 16,
	RP         = 17,
	AFSB       = 18,
	X25        = 19,
	ISDN       = 20,
	RT         = 21,
	Nsap       = 22,
	NsapPtr    = 23,
	Sig        = 24,
	Key        = 25,
	PX         = 26,
	GPos       = 27,
	AAAA       = 28,
	Loc        = 29,
	NXT        = 30,
	EID        = 31,
	NimLoc     = 32,
	Srv        = 33,
	ATMA       = 34,
	NaPtr      = 35,
	KX         = 36,
	Cert       = 37,
	A6         = 38,
	DName      = 39,
	SINK       = 40,
	OPT        = 41,
	APL        = 42,
	DS         = 43,
	SSHFP      = 44,
	IpSecKey   = 45,
	RRSig      = 46,
	NSec       = 47,
	DnsKey     = 48,
	DHCID      = 49,
	NSec3      = 50,
	NSec3Param = 51,
	TLSA       = 52,
	SMIMEA     = 53,
	HIP        = 55,
	NInfo      = 56,
	RKey       = 57,
	TALink     = 58,
	CDS        = 59,
	CDNSKey    = 60,
	OpenGPGKey = 61,
	CSYNC      = 62,
	ZoneMD     = 63,
	SVCB       = 64,
	HTTPS      = 65,
	SPF        = 99,
	UInfo      = 100,
	UID        = 101,
	GID        = 102,
	UNSPEC     = 103,
	NID        = 104,
	L32        = 105,
	L64        = 106,
	LP         = 107,
	EUI48      = 108,
	EUI64      = 109,
	TKey       = 249,
	TSig       = 250,
	AXFR       = 252,
	MailB      = 253,
	MailA      = 254,
	All        = 255,
	URI        = 256,
	CAA        = 257,
	AVC        = 258,
	DOA        = 259,
	AMTRELAY   = 260,
}

impl TryFrom<u16> for Type {
	type Error = ();
	
	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			1..=53 | 55..=65 | 99..=109 | 249..=260 => Ok(unsafe { std::mem::transmute(value) }),
			_ => Err(())
		}
	}
}

impl FromStr for Type {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"A"          => Self::A,
			"NS"         => Self::NS,
			"MD"         => Self::MD,
			"MF"         => Self::MF,
			"CNAME"      => Self::CName,
			"SOA"        => Self::SOA,
			"MB"         => Self::MB,
			"MG"         => Self::MG,
			"MR"         => Self::MR,
			"NULL"       => Self::Null,
			"WKS"        => Self::WKS,
			"PTR"        => Self::Ptr,
			"HINFO"      => Self::HInfo,
			"MINFO"      => Self::MInfo,
			"MX"         => Self::MX,
			"TXT"        => Self::TXT,
			"RP"         => Self::RP,
			"X25"        => Self::X25,
			"ISDN"       => Self::ISDN,
			"RT"         => Self::RT,
			"NSAP"       => Self::Nsap,
			"NSAP_PTR"   => Self::Ptr,
			"SIG"        => Self::Sig,
			"KEY"        => Self::Key,
			"PX"         => Self::PX,
			"GPOS"       => Self::GPos,
			"AAAA"       => Self::AAAA,
			"LOC"        => Self::Loc,
			"NXT"        => Self::NXT,
			"EID"        => Self::EID,
			"NIMLOC"     => Self::NimLoc,
			"SRV"        => Self::Srv,
			"ATMA"       => Self::ATMA,
			"NAP_PTR"    => Self::Ptr,
			"KX"         => Self::KX,
			"CERT"       => Self::Cert,
			"A6"         => Self::A6,
			"DNAME"      => Self::DName,
			"SINK"       => Self::SINK,
			"OPT"        => Self::OPT,
			"APL"        => Self::APL,
			"DS"         => Self::DS,
			"SSHFP"      => Self::SSHFP,
			"IPSECKEY"   => Self::IpSecKey,
			"RRSIG"      => Self::RRSig,
			"NSEC"       => Self::NSec,
			"DNSKEY"     => Self::DnsKey,
			"DHCID"      => Self::DHCID,
			"NSEC3"      => Self::NSec3,
			"NSEC3PARAM" => Self::NSec3Param,
			"TLSA"       => Self::TLSA,
			"SMIMEA"     => Self::SMIMEA,
			"HIP"        => Self::HIP,
			"NINFO"      => Self::NInfo,
			"RKEY"       => Self::RKey,
			"TALINK"     => Self::TALink,
			"CDS"        => Self::CDS,
			"CDNSKEY"    => Self::CDNSKey,
			"OPENGPGKEY" => Self::OpenGPGKey,
			"CSYNC"      => Self::CSYNC,
			"ZONEMD"     => Self::ZoneMD,
			"SVCB"       => Self::SVCB,
			"HTTPS"      => Self::HTTPS,
			"SPF"        => Self::SPF,
			"UINFO"      => Self::UInfo,
			"UID"        => Self::UID,
			"GID"        => Self::GID,
			"UNSPEC"     => Self::UNSPEC,
			"NID"        => Self::NID,
			"L32"        => Self::L32,
			"L64"        => Self::L64,
			"LP"         => Self::LP,
			"EUI48"      => Self::EUI48,
			"EUI64"      => Self::EUI64,
			"TKEY"       => Self::TKey,
			"TSIG"       => Self::TSig,
			"AXFR"       => Self::AXFR,
			"MAILB"      => Self::MailB,
			"MAILA"      => Self::MailA,
			"*"          => Self::All,
			"URI"        => Self::URI,
			"CAA"        => Self::CAA,
			"AVC"        => Self::AVC,
			"DOA"        => Self::DOA,
			"AMTRELAY"   => Self::AMTRELAY,
			_            => return Err(ZoneParseError::new(ZoneParseErrorType::RecordType, 0, 0))
		})
	}
}

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(match self {
			Self::A          => "A",
			Self::NS         => "NS",
			Self::MD         => "MD",
			Self::MF         => "MF",
			Self::CName      => "CNAME",
			Self::SOA        => "SOA",
			Self::MB         => "MB",
			Self::MG         => "MG",
			Self::MR         => "MR",
			Self::Null       => "NULL",
			Self::WKS        => "WKS",
			Self::Ptr        => "PTR",
			Self::HInfo      => "HINFO",
			Self::MInfo      => "MINFO",
			Self::MX         => "MX",
			Self::TXT        => "TXT",
			Self::RP         => "RP",
			Self::AFSB       => "AFSB",
			Self::X25        => "X25",
			Self::ISDN       => "ISDN",
			Self::RT         => "RT",
			Self::Nsap       => "NSAP",
			Self::NsapPtr    => "NSAP-PTR",
			Self::Sig        => "SIG",
			Self::Key        => "KEY",
			Self::PX         => "PX",
			Self::GPos       => "GPOS",
			Self::AAAA       => "AAAA",
			Self::Loc        => "LOC",
			Self::NXT        => "NXT",
			Self::EID        => "EID",
			Self::NimLoc     => "NIMLOC",
			Self::Srv        => "SRV",
			Self::ATMA       => "ATMA",
			Self::NaPtr      => "NAPTR",
			Self::KX         => "KX",
			Self::Cert       => "CERT",
			Self::A6         => "A6",
			Self::DName      => "DNAME",
			Self::SINK       => "SINK",
			Self::OPT        => "OPT",
			Self::APL        => "APL",
			Self::DS         => "DS",
			Self::SSHFP      => "SSHFP",
			Self::IpSecKey   => "IPSECKEY",
			Self::RRSig      => "RRSIG",
			Self::NSec       => "NSEC",
			Self::DnsKey     => "DNSKEY",
			Self::DHCID      => "DHCID",
			Self::NSec3      => "NSEC3",
			Self::NSec3Param => "NSEC3PARAM",
			Self::TLSA       => "TLSA",
			Self::SMIMEA     => "SMIMEA",
			Self::HIP        => "HIP",
			Self::NInfo      => "NINFO",
			Self::RKey       => "RKEY",
			Self::TALink     => "TALINK",
			Self::CDS        => "CDS",
			Self::CDNSKey    => "CDNSKEY",
			Self::OpenGPGKey => "OPENGPGKEY",
			Self::CSYNC      => "CSYNC",
			Self::ZoneMD     => "ZONEMD",
			Self::SVCB       => "SVCB",
			Self::HTTPS      => "HTTPS",
			Self::SPF        => "SPF",
			Self::UInfo      => "UINFO",
			Self::UID        => "UID",
			Self::GID        => "GID",
			Self::UNSPEC     => "UNSPEC",
			Self::NID        => "NID",
			Self::L32        => "L32",
			Self::L64        => "L64",
			Self::LP         => "LP",
			Self::EUI48      => "EUI48",
			Self::EUI64      => "EUI64",
			Self::TKey       => "TKEY",
			Self::TSig       => "TSIG",
			Self::AXFR       => "AXFR",
			Self::MailB      => "MAILB",
			Self::MailA      => "MAILA",
			Self::All        => "*",
			Self::URI        => "URI",
			Self::CAA        => "CAA",
			Self::AVC        => "AVC",
			Self::DOA        => "DOA",
			Self::AMTRELAY   => "AMTRELAY",
		})
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SoaRecord {
	pub name_server: DomainName,
	pub mailbox:     DomainName,
	pub serial:      u32,
	pub refresh:     u32,
	pub retry:       u32,
	pub expire:      u32,
	pub minimum:     u32
}

impl FromStr for SoaRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for SoaRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WksRecord {
	pub address:  std::net::Ipv4Addr,
	pub protocol: u8,
	pub data:     Vec<u8>
}

impl FromStr for WksRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for WksRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HardwareInfoRecord {
	pub cpu: String,
	pub os:  String
}

impl HardwareInfoRecord {
	pub fn get() -> Self {
		Self {
			cpu: std::env::consts::ARCH.to_string(),
			os:  std::env::consts::OS.to_string()
		}
	}
}

impl FromStr for HardwareInfoRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for HardwareInfoRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailInfoRecord {
	pub responsible: DomainName,
	pub error:       DomainName
}

impl FromStr for MailInfoRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for MailInfoRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailExchangeRecord {
	pub preference: u16,
	pub exchange:   DomainName
}

impl FromStr for MailExchangeRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for MailExchangeRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResponsiblePerson {
	pub mailbox: DomainName,
	pub txt:     DomainName
}

impl FromStr for ResponsiblePerson {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for ResponsiblePerson {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IsdnRecord {
	pub address:     String,
	pub sub_address: String
}

impl FromStr for IsdnRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for IsdnRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RouteThroughRecord {
	pub preference:        u16,
	pub intermediate_host: DomainName
}

impl FromStr for RouteThroughRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for RouteThroughRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NsapRecord(pub Vec<u8>);

impl FromStr for NsapRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NsapRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("0x")?;
		
		for b in &self.0 {
			write!(f, "{:02x}", b)?;
		}
		
		Ok(())
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AFSDataBaseRecord {
	pub subtype:  u16,
	pub hostname: DomainName
}

impl FromStr for AFSDataBaseRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for AFSDataBaseRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SignatureRecord {
	pub type_covered:         u16,
	pub algorithm:            u8,
	pub labels:               u8,
	pub original_ttl:         u32,
	pub signature_expiration: u32,
	pub signature_inception:  u32,
	pub key_tag:              u16,
	pub signer_name:          DomainName,
	pub signature:            Vec<u8>
}

impl FromStr for SignatureRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for SignatureRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyRecord {
	pub flags:     u16,
	pub protocol:   u8,
	pub algorithm:  u8,
	pub public_key: Vec<u8>
}

impl FromStr for KeyRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for KeyRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PxRecord {
	pub preference: u16,
	pub map822:     DomainName,
	pub map_x400:   DomainName
}

impl FromStr for PxRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for PxRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GPosRecord {
	pub longitude: String,
	pub latitude:  String,
	pub altitude:  String
}

impl FromStr for GPosRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for GPosRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationRecord {
	pub version:              u8,
	pub size:                 u8,
	pub horizontal_precision: u8,
	pub vertical_precision:   u8,
	pub latitude:             u32,
	pub longitude:            u32,
	pub altitude:             u32
}

impl FromStr for LocationRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for LocationRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NxtRecord {
	pub next_domain_name: DomainName,
	pub type_bit_map:     Vec<u8>
}

impl FromStr for NxtRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NxtRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SrvRecord {
	pub priority: u16,
	pub weight:   u16,
	pub port:     u16,
	pub target:   DomainName
}

impl FromStr for SrvRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for SrvRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NaPtr {
	pub order:      u16,
	pub preference:  u16,
	pub flags:       String,
	pub service:     Vec<String>,
	pub regexp:      String,
	pub replacement: DomainName
}

impl FromStr for NaPtr {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NaPtr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyExchangeRecord {
	pub preference: u16,
	pub exchange:   DomainName
}

impl FromStr for KeyExchangeRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for KeyExchangeRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CertRecord {
	pub r#type:      u16,
	pub key_tag:     u16,
	pub algorithm:   u8,
	pub certificate: Vec<u8>
}

impl FromStr for CertRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for CertRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct A6Record {
	pub prefix_len: u8,
	pub address:    std::net::Ipv6Addr,
	pub prefix:     DomainName
}

impl FromStr for A6Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for A6Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SinkRecord {
	pub coding:    u8,
	pub subcoding: u8,
	pub data:      Vec<u8>
}

impl FromStr for SinkRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for SinkRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AplRecord {
	address_family: u16,
	prefix:         u8,
	afd_length:     i8,
	afd_part:       Vec<u8>
}

impl FromStr for AplRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for AplRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DsRecord {
	pub key_tag:     u16,
	pub algorithm:   u8,
	pub digest_type: u8,
	pub digest:      Vec<u8>
}

impl FromStr for DsRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for DsRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SshFpRecord {
	pub algorithm:   u8,
	pub fp_type:     u8,
	pub fingerprint: Vec<u8>
}

impl FromStr for SshFpRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for SshFpRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpSecKeyRecord {
	pub precedence:   u8,
	pub algorithm:    u8,
	pub gateway:      IpSecKeyRecordGateway,
	pub public_key:   Vec<u8>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpSecKeyRecordGateway {
	IPv4(std::net::Ipv4Addr),
	IPv6(std::net::Ipv6Addr),
	Domain(DomainName)
}

impl FromStr for IpSecKeyRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for IpSecKeyRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RRSigRecord {
	pub tye_covered:          u16,
	pub algorithm:            u8,
	pub labels:               u8,
	pub original_ttl:         u32,
	pub signature_expiration: u32,
	pub signature_inception:  u32,
	pub key_tag:              u16,
	pub signer_name:          String,
	pub signature:            Vec<u8>
}

impl FromStr for RRSigRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for RRSigRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NSecRecord {
	pub next_domain_name: DomainName,
	pub type_bit_maps:    Vec<u8>
}

impl FromStr for NSecRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NSecRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DnsKeyRecord {
	pub flags:      u16,
	pub protocol:   u8,
	pub algorithm:  u8,
	pub public_key: Vec<u8>
}

impl FromStr for DnsKeyRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for DnsKeyRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DHCIDRecord {
	pub identifier_type: u16,
	pub digest_type:     u8,
	pub digest:          Vec<u8>
}

impl FromStr for DHCIDRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for DHCIDRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NSec3Record {

}

impl FromStr for NSec3Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NSec3Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NSec3ParamRecord {

}

impl FromStr for NSec3ParamRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NSec3ParamRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TlsaRecord {
	pub cert_usage:      u8,
	pub selector:        u8,
	pub matching_type:   u8,
	pub cert_assoc_data: Vec<u8>
}

impl FromStr for TlsaRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for TlsaRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HipRecord {

}

impl FromStr for HipRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for HipRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RKeyRecord {
	pub flags:      u16,
	pub protocol:   u8,
	pub algorithm:  u8,
	pub public_key: Vec<u8>
}

impl FromStr for RKeyRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for RKeyRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TALinkRecord {
	pub prev: DomainName,
	pub next: DomainName
}

impl FromStr for TALinkRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for TALinkRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CSyncRecord {
	pub soa_serial:   u32,
	pub flags:        u16,
	pub type_bit_map: Vec<u8>
}

impl FromStr for CSyncRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for CSyncRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoneMDRecord {
	pub serial: u32,
	pub scheme: u8,
	pub algo:   u8,
	pub digest: Vec<u8>
}

impl FromStr for ZoneMDRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for ZoneMDRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NIDRecord {
	pub preference: u16,
	pub node_id:    u64
}

impl FromStr for NIDRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NIDRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct L32Record {
	pub preference: u16,
	pub locator:    u32
}

impl FromStr for L32Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for L32Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct L64Record {
	pub preference: u16,
	pub locator:    u64
}

impl FromStr for L64Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for L64Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LPRecord {
	pub preference: u16,
	pub fqdn:       Vec<u8>
}

impl FromStr for LPRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for LPRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TSigRecord {

}

impl FromStr for TSigRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for TSigRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct EUI48Record([u8; 6]);

impl FromStr for EUI48Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for EUI48Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct EUI64Record([u8; 8]);

impl FromStr for EUI64Record {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for EUI64Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TKeyRecord {

}

impl FromStr for TKeyRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for TKeyRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UriRecord {
	pub priority: u16,
	pub weight:   u16,
	pub target:   String
}

impl FromStr for UriRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for UriRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaaRecord {
	pub flags: u8,
	pub tag:   String,
	pub value: Vec<u8>
}

impl FromStr for CaaRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for CaaRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmtRelayRecord {
	pub precedence: u8,
	pub relay:      AmtRelayRecordRelay
}

impl FromStr for AmtRelayRecord {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for AmtRelayRecord {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NimrodLocator(pub Vec<u8>);

impl FromStr for NimrodLocator {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		unimplemented!()
	}
}

impl fmt::Display for NimrodLocator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		unimplemented!()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmtRelayRecordRelay {
	Empty,
	IPv4(std::net::Ipv4Addr),
	IPv6(std::net::Ipv6Addr),
	Domain(DomainName)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnimplementedRecord {
	Wire { id: u16, data: Box<[u8]> },
	Text { name: String, data: String }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceRecordData {
	A(std::net::Ipv4Addr),
	NS(DomainName),
	MD(DomainName),
	MF(DomainName),
	CName(DomainName),
	SOA(SoaRecord),
	MB(DomainName),
	MG(DomainName),
	MR(DomainName),
	Null(Vec<u8>),
	WKS(WksRecord),
	Ptr(DomainName),
	HInfo(HardwareInfoRecord),
	MInfo(MailInfoRecord),
	MX(MailExchangeRecord),
	TXT(String),
	RP(ResponsiblePerson),
	AFSDB(AFSDataBaseRecord),
	X25(String),
	ISDN(IsdnRecord),
	RT(RouteThroughRecord),
	Nsap(NsapRecord),
	NsapPtr(DomainName),
	Sig(SignatureRecord),
	Key(KeyRecord),
	PX(PxRecord),
	GPos(GPosRecord),
	AAAA(std::net::Ipv6Addr),
	Loc(LocationRecord),
	NXT(NxtRecord),
	EID(NimrodLocator),
	NimLoc(NimrodLocator),
	Srv(SrvRecord),
	ATMA,
	NaPtr(NaPtr),
	KX(KeyExchangeRecord),
	Cert(CertRecord),
	A6(A6Record),
	DName(DomainName),
	Sink(SinkRecord),
	OPT,
	APL(AplRecord),
	DS(DsRecord),
	SSHFP(SshFpRecord),
	IpSecKey(IpSecKeyRecord),
	RRSig(RRSigRecord),
	NSec(NSecRecord),
	DnsKey(DnsKeyRecord),
	DHCID(DHCIDRecord),
	NSec3(NSec3Record),
	NSec3Param(NSec3ParamRecord),
	TLSA(TlsaRecord),
	SMIMEA(TlsaRecord),
	HIP(HipRecord),
	NInfo(String),
	RKey(RKeyRecord),
	TALink(TALinkRecord),
	CDS(DsRecord),
	CDNSKey(DnsKeyRecord),
	OpenGPGKey(Vec<u8>),
	CSYNC(CSyncRecord),
	ZoneMD(ZoneMDRecord),
	SVCB(Vec<u8>),
	HTTPS(Vec<u8>),
	SPF(String),
	UInfo,
	UID,
	GID,
	UNSPEC,
	NID(NIDRecord),
	L32(L32Record),
	L64(L64Record),
	LP(LPRecord),
	EUI48(EUI48Record),
	EUI64(EUI64Record),
	TKey(TKeyRecord),
	TSig(TSigRecord),
	AXFR,
	MailB,
	MailA,
	All,
	URI(UriRecord),
	CAA(CaaRecord),
	AVC,
	DOA,
	AMTRELAY(AmtRelayRecord),
	Other(UnimplementedRecord)
}

impl ResourceRecordData {
	pub fn to_type(&self) -> Result<Type, Option<u16>> {
		Ok(match self {
			Self::A(_)          => Type::A,
			Self::NS(_)         => Type::NS,
			Self::MD(_)         => Type::MD,
			Self::MF(_)         => Type::MF,
			Self::CName(_)      => Type::CName,
			Self::SOA(_)        => Type::SOA,
			Self::MB(_)         => Type::MB,
			Self::MG(_)         => Type::MG,
			Self::MR(_)         => Type::MR,
			Self::Null(_)       => Type::Null,
			Self::WKS(_)        => Type::WKS,
			Self::Ptr(_)        => Type::Ptr,
			Self::HInfo(_)      => Type::HInfo,
			Self::MInfo(_)      => Type::MInfo,
			Self::MX(_)         => Type::MX,
			Self::TXT(_)        => Type::TXT,
			Self::RP(_)         => Type::RP,
			Self::AFSDB(_)      => Type::AFSB,
			Self::X25(_)        => Type::X25,
			Self::ISDN(_)       => Type::ISDN,
			Self::RT(_)         => Type::RT,
			Self::Nsap(_)       => Type::Nsap,
			Self::NsapPtr(_)    => Type::NsapPtr,
			Self::Sig(_)        => Type::Sig,
			Self::Key(_)        => Type::Key,
			Self::PX(_)         => Type::PX,
			Self::GPos(_)       => Type::GPos,
			Self::AAAA(_)       => Type::AAAA,
			Self::Loc(_)        => Type::Loc,
			Self::NXT(_)        => Type::NXT,
			Self::EID(_)        => Type::EID,
			Self::NimLoc(_)     => Type::NimLoc,
			Self::Srv(_)        => Type::Srv,
			Self::ATMA          => Type::ATMA,
			Self::NaPtr(_)      => Type::NaPtr,
			Self::KX(_)         => Type::KX,
			Self::Cert(_)       => Type::Cert,
			Self::A6(_)         => Type::A6,
			Self::DName(_)      => Type::DName,
			Self::Sink(_)       => Type::SINK,
			Self::OPT           => Type::OPT,
			Self::APL(_)        => Type::APL,
			Self::DS(_)         => Type::DS,
			Self::SSHFP(_)      => Type::SSHFP,
			Self::IpSecKey(_)   => Type::IpSecKey,
			Self::RRSig(_)      => Type::RRSig,
			Self::NSec(_)       => Type::NSec,
			Self::DnsKey(_)     => Type::DnsKey,
			Self::DHCID(_)      => Type::DHCID,
			Self::NSec3(_)      => Type::NSec3,
			Self::NSec3Param(_) => Type::NSec3Param,
			Self::TLSA(_)       => Type::TLSA,
			Self::SMIMEA(_)     => Type::SMIMEA,
			Self::HIP(_)        => Type::HIP,
			Self::NInfo(_)      => Type::NInfo,
			Self::RKey(_)       => Type::RKey,
			Self::TALink(_)     => Type::TALink,
			Self::CDS(_)        => Type::CDS,
			Self::CDNSKey(_)    => Type::CDNSKey,
			Self::OpenGPGKey(_) => Type::OpenGPGKey,
			Self::CSYNC(_)      => Type::CSYNC,
			Self::ZoneMD(_)     => Type::ZoneMD,
			Self::SVCB(_)       => Type::SVCB,
			Self::HTTPS(_)      => Type::HTTPS,
			Self::SPF(_)        => Type::SPF,
			Self::UInfo         => Type::UInfo,
			Self::UID           => Type::UID,
			Self::GID           => Type::GID,
			Self::UNSPEC        => Type::UNSPEC,
			Self::NID(_)        => Type::NID,
			Self::L32(_)        => Type::L32,
			Self::L64(_)        => Type::L64,
			Self::LP(_)         => Type::LP,
			Self::EUI48(_)      => Type::EUI48,
			Self::EUI64(_)      => Type::EUI64,
			Self::TKey(_)       => Type::TKey,
			Self::TSig(_)       => Type::TSig,
			Self::AXFR          => Type::AXFR,
			Self::MailB         => Type::MailB,
			Self::MailA         => Type::MailA,
			Self::All           => Type::All,
			Self::URI(_)        => Type::URI,
			Self::CAA(_)        => Type::CAA,
			Self::AVC           => Type::AVC,
			Self::DOA           => Type::DOA,
			Self::AMTRELAY(_)   => Type::AMTRELAY,
			Self::Other(v)      => return Err(match v {
				UnimplementedRecord::Wire { id, .. } => Some(*id),
				UnimplementedRecord::Text { .. }     => None
			})
		})
	}
	
	pub(super) fn read(ty: u16, data: &[u8]) -> io::Result<Self> {
		unimplemented!()
	}
	
	pub(super) fn write(&self, buf: &mut Vec<u8>) -> u16 {
		unimplemented!();
	}
}

impl FromStr for ResourceRecordData {
	type Err = ZoneParseError;
	
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (ty, s) = s.split_once(' ').ok_or_else(|| ZoneParseError::new(ZoneParseErrorType::RecordType, 0, 0))?;
		let row = ty.len();
		
		Ok(match ty.parse()? {
			Type::A          => Self::A(s.parse()
				.map_err(|_| ZoneParseError::new(ZoneParseErrorType::RecordValue, 0, row))?),
			Type::NS         => Self::NS(s.to_string()),
			Type::MD         => Self::MD(s.to_string()),
			Type::MF         => Self::MF(s.to_string()),
			Type::CName      => Self::CName(s.to_string()),
			Type::SOA        => Self::SOA(s.parse()?),
			Type::MB         => Self::MB(s.to_string()),
			Type::MG         => Self::MG(s.to_string()),
			Type::MR         => Self::MR(s.to_string()),
			Type::WKS        => Self::WKS(s.parse()?),
			Type::Ptr        => Self::Ptr(s.to_string()),
			Type::HInfo      => Self::HInfo(s.parse()?),
			Type::MInfo      => Self::MInfo(s.parse()?),
			Type::MX         => Self::MX(s.parse()?),
			Type::TXT        => Self::TXT(s.to_string()),
			Type::RP         => Self::RP(s.parse()?),
			Type::X25        => Self::X25(s.to_string()),
			Type::ISDN       => Self::ISDN(s.parse()?),
			Type::RT         => Self::RT(s.parse()?),
			Type::Nsap       => Self::Nsap(s.parse()?),
			Type::NsapPtr    => Self::NsapPtr(s.to_string()),
			Type::Sig        => Self::Sig(s.parse()?),
			Type::Key        => Self::Key(s.parse()?),
			Type::PX         => Self::PX(s.parse()?),
			Type::GPos       => Self::GPos(s.parse()?),
			Type::AAAA       => Self::AAAA(s.parse()
				.map_err(|_| ZoneParseError::new(ZoneParseErrorType::RecordValue, 0, row))?),
			Type::Loc        => Self::Loc(s.parse()?),
			Type::NXT        => Self::NXT(s.parse()?),
			Type::EID        => Self::EID(s.parse()?),
			Type::NimLoc     => Self::NimLoc(s.parse()?),
			Type::Srv        => Self::Srv(s.parse()?),
			Type::NaPtr      => Self::NaPtr(s.parse()?),
			Type::KX         => Self::KX(s.parse()?),
			Type::Cert       => Self::Cert(s.parse()?),
			Type::A6         => Self::A6(s.parse()?),
			Type::DName      => Self::DName(s.to_string()),
			Type::SINK       => Self::Sink(s.parse()?),
			Type::APL        => Self::APL(s.parse()?),
			Type::DS         => Self::DS(s.parse()?),
			Type::SSHFP      => Self::SSHFP(s.parse()?),
			Type::IpSecKey   => Self::IpSecKey(s.parse()?),
			Type::RRSig      => Self::RRSig(s.parse()?),
			Type::NSec       => Self::NSec(s.parse()?),
			Type::DnsKey     => Self::DnsKey(s.parse()?),
			Type::DHCID      => Self::DHCID(s.parse()?),
			Type::NSec3      => Self::NSec3(s.parse()?),
			Type::NSec3Param => Self::NSec3Param(s.parse()?),
			Type::TLSA       => Self::TLSA(s.parse()?),
			Type::SMIMEA     => Self::SMIMEA(s.parse()?),
			Type::HIP        => Self::HIP(s.parse()?),
			Type::NInfo      => Self::NInfo(s.to_string()),
			Type::RKey       => Self::RKey(s.parse()?),
			Type::TALink     => Self::TALink(s.parse()?),
			Type::CDS        => Self::CDS(s.parse()?),
			Type::CDNSKey    => Self::CDNSKey(s.parse()?),
			Type::OpenGPGKey => Self::OpenGPGKey(base64::decode(s)
				.map_err(|_| ZoneParseError::new(ZoneParseErrorType::RecordValue, 0, row))?),
			Type::CSYNC      => Self::CSYNC(s.parse()?),
			Type::ZoneMD     => Self::ZoneMD(s.parse()?),
			Type::SVCB       => Self::SVCB(base64::decode(s)
				.map_err(|_| ZoneParseError::new(ZoneParseErrorType::RecordValue, 0, row))?),
			Type::HTTPS      => Self::HTTPS(base64::decode(s)
				.map_err(|_| ZoneParseError::new(ZoneParseErrorType::RecordValue, 0, row))?),
			Type::SPF        => Self::SPF(s.to_string()),
			Type::NID        => Self::NID(s.parse()?),
			Type::L32        => Self::L32(s.parse()?),
			Type::L64        => Self::L64(s.parse()?),
			Type::LP         => Self::LP(s.parse()?),
			Type::EUI48      => Self::EUI48(s.parse()?),
			Type::EUI64      => Self::EUI64(s.parse()?),
			Type::TKey       => Self::TKey(s.parse()?),
			Type::TSig       => Self::TSig(s.parse()?),
			Type::All        => Self::All,
			Type::URI        => Self::URI(s.parse()?),
			Type::CAA        => Self::CAA(s.parse()?),
			Type::AMTRELAY   => Self::AMTRELAY(s.parse()?),
			ty               => Self::Other(UnimplementedRecord::Text {
				name: ty.to_string(),
				data: s.to_string()
			})
		})
	}
}

impl fmt::Display for ResourceRecordData {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ResourceRecordData::A(v)            => write!(f, "A {}", v),
			ResourceRecordData::NS(v)           => write!(f, "NS {}", v),
			ResourceRecordData::MD(v)           => write!(f, "MD {}", v),
			ResourceRecordData::MF(v)           => write!(f, "MF {}", v),
			ResourceRecordData::CName(v)        => write!(f, "CNAME {}", v),
			ResourceRecordData::SOA(v)          => write!(f, "SOA {}", v),
			ResourceRecordData::MB(v)           => write!(f, "MB {}", v),
			ResourceRecordData::MG(v)           => write!(f, "MG {}", v),
			ResourceRecordData::MR(v)           => write!(f, "MR {}", v),
			ResourceRecordData::WKS(v)          => write!(f, "WKS {}", v),
			ResourceRecordData::Ptr(v)          => write!(f, "PTR {}", v),
			ResourceRecordData::HInfo(v)        => write!(f, "HINFO {}", v),
			ResourceRecordData::MInfo(v)        => write!(f, "MINFO {}", v),
			ResourceRecordData::MX(v)           => write!(f, "MX {}", v),
			ResourceRecordData::TXT(v)          => write!(f, "TXT {}", v),
			ResourceRecordData::RP(v)           => write!(f, "RP {}", v),
			ResourceRecordData::AFSDB(v)        => write!(f, "AFSDB {}", v),
			ResourceRecordData::X25(v)          => write!(f, "X25 {}", v),
			ResourceRecordData::ISDN(v)         => write!(f, "ISDN {}", v),
			ResourceRecordData::RT(v)           => write!(f, "RT {}", v),
			ResourceRecordData::Nsap(v)         => write!(f, "NSAP 0x{}", v),
			ResourceRecordData::NsapPtr(v)      => write!(f, "NSAP_PTR {}", v),
			ResourceRecordData::Sig(v)          => write!(f, "SIG {}", v),
			ResourceRecordData::Key(v)          => write!(f, "KEY {}", v),
			ResourceRecordData::PX(v)           => write!(f, "PX {}", v),
			ResourceRecordData::GPos(v)         => write!(f, "GPOS {}", v),
			ResourceRecordData::AAAA(v)         => write!(f, "AAAA {}", v),
			ResourceRecordData::Loc(v)          => write!(f, "LOC {}", v),
			ResourceRecordData::NXT(v)          => write!(f, "NXT {}", v),
			ResourceRecordData::EID(v)          => write!(f, "EID {}", v),
			ResourceRecordData::NimLoc(v)       => write!(f, "NIMLOC {}", v),
			ResourceRecordData::Srv(v)          => write!(f, "SRV {}", v),
			ResourceRecordData::NaPtr(v)        => write!(f, "NA_PTR {}", v),
			ResourceRecordData::KX(v)           => write!(f, "KX {}", v),
			ResourceRecordData::Cert(v)         => write!(f, "CERT {}", v),
			ResourceRecordData::A6(v)           => write!(f, "A6 {}", v),
			ResourceRecordData::DName(v)        => write!(f, "DNAME {}", v),
			ResourceRecordData::Sink(v)         => write!(f, "SINK {}", v),
			ResourceRecordData::APL(v)          => write!(f, "APL {}", v),
			ResourceRecordData::DS(v)           => write!(f, "DS {}", v),
			ResourceRecordData::SSHFP(v)        => write!(f, "SSHFP {}", v),
			ResourceRecordData::IpSecKey(v)     => write!(f, "IPSECKEY {}", v),
			ResourceRecordData::RRSig(v)        => write!(f, "RRSIG {}", v),
			ResourceRecordData::NSec(v)         => write!(f, "NSEC {}", v),
			ResourceRecordData::DnsKey(v)       => write!(f, "DNSKEY {}", v),
			ResourceRecordData::DHCID(v)        => write!(f, "DHCID {}", v),
			ResourceRecordData::NSec3(v)        => write!(f, "NSEC3 {}", v),
			ResourceRecordData::NSec3Param(v)   => write!(f, "NSEC3PARAM {}", v),
			ResourceRecordData::TLSA(v)         => write!(f, "TLSA {}", v),
			ResourceRecordData::SMIMEA(v)       => write!(f, "SMIMEA {}", v),
			ResourceRecordData::HIP(v)          => write!(f, "HIP {}", v),
			ResourceRecordData::NInfo(v)        => write!(f, "NINFO {}", v),
			ResourceRecordData::RKey(v)         => write!(f, "RKEY {}", v),
			ResourceRecordData::TALink(v)       => write!(f, "TALINK {}", v),
			ResourceRecordData::CDS(v)          => write!(f, "CDS {}", v),
			ResourceRecordData::CDNSKey(v)      => write!(f, "CDNSKEY {}", v),
			ResourceRecordData::OpenGPGKey(v)   => write!(f, "OPENGPGKEY {}", base64::encode(v)),
			ResourceRecordData::CSYNC(v)        => write!(f, "CSYNC {}", v),
			ResourceRecordData::ZoneMD(v)       => write!(f, "ZONEMD {}", v),
			ResourceRecordData::SVCB(v)         => write!(f, "SVCB {}", base64::encode(v)),
			ResourceRecordData::HTTPS(v)        => write!(f, "HTTPS {}", base64::encode(v)),
			ResourceRecordData::SPF(v)          => write!(f, "SPF {}", v),
			ResourceRecordData::NID(v)          => write!(f, "NID {}", v),
			ResourceRecordData::L32(v)          => write!(f, "L32 {}", v),
			ResourceRecordData::L64(v)          => write!(f, "L64 {}", v),
			ResourceRecordData::LP(v)           => write!(f, "LP {}", v),
			ResourceRecordData::EUI48(v)        => write!(f, "EUI48 {}", v),
			ResourceRecordData::EUI64(v)        => write!(f, "EUI64 {}", v),
			ResourceRecordData::TKey(v)         => write!(f, "TKey {}", v),
			ResourceRecordData::TSig(v)         => write!(f, "TSig {}", v),
			ResourceRecordData::All             => f.write_str("*"),
			ResourceRecordData::URI(v)          => write!(f, "URI {}", v),
			ResourceRecordData::CAA(v)          => write!(f, "CAA {}", v),
			ResourceRecordData::AMTRELAY(v)     => write!(f, "AMTRELAY {}", v),
			ResourceRecordData::Other(UnimplementedRecord::Text { name, data }) => write!(f, "{} {}", name, data),
			_                                   => unimplemented!("non-standardized RR")
		}
	}
}