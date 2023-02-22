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

use std::cell::UnsafeCell;
use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ExemplarFilter {
	None,
	All,
	WithSampledTrace
}

impl Default for ExemplarFilter {
	fn default() -> Self {
		Self::WithSampledTrace
	}
}

impl std::str::FromStr for ExemplarFilter {
	type Err = ();
	
	fn from_str(v: &str) -> Result<Self, Self::Err> {
		Ok(match v {
			"all"                => Self::All,
			"none"               => Self::None,
			"with_sampled_trace" => Self::WithSampledTrace,
			_                    => return Err(())
		})
	}
}

#[derive(Clone, Debug)]
pub struct InstrumentParameters {
	pub name:        Cow<'static, str>,
	pub unit:        Option<Cow<'static, str>>,
	pub desc:        Option<Cow<'static, str>>,
	pub aggregation: Aggregation
}

impl Default for InstrumentParameters {
	fn default() -> Self {
		Self {
			name:        Default::default(),
			unit:        None,
			desc:        None,
			aggregation: Aggregation::LastValue
		}
	}
}

impl InstrumentParameters {
	pub fn new() -> Self {
		Self::default()
	}
	
	pub fn name(mut self, v: String) -> Self {
		self.name = Cow::Owned(v);
		self
	}
	
	pub fn name_str(mut self, v: &'static str) -> Self {
		self.name = Cow::Borrowed(v);
		self
	}
	
	pub fn unit(mut self, v: Cow<'static, str>) -> Self {
		self.unit = Some(v);
		self
	}
	
	pub fn desc(mut self, v: Cow<'static, str>) -> Self {
		self.desc = Some(v);
		self
	}
	
	pub fn aggregation(mut self, v: Aggregation) -> Self {
		self.aggregation = v;
		self
	}
	
	pub fn aggregation_drop(mut self) -> Self {
		self.aggregation = Aggregation::Drop;
		self
	}
	
	pub fn aggregation_sum(mut self, temporality: AggregationTemporality, monotonic: bool) -> Self {
		self.aggregation = Aggregation::Sum { temporality, monotonic };
		self
	}
	
	pub fn aggregation_last_value(mut self) -> Self {
		self.aggregation = Aggregation::LastValue;
		self
	}
	
	pub fn aggregation_histogram(mut self, temporality: AggregationTemporality) -> Self {
		self.aggregation = Aggregation::Histogram { temporality };
		self
	}
}

#[derive(Clone, Debug)]
pub enum Aggregation {
	Drop,
	Sum { temporality: AggregationTemporality, monotonic: bool },
	LastValue,
	Histogram { temporality: AggregationTemporality }
}

pub trait DataPointValue {
	const TYPE: DataPointType;
	
	fn into(self) -> NumberDataPointValue;
}

impl DataPointValue for u8 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for u16 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for u32 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for u64 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for usize {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for i8 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for i16 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for i32 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for i64 {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for isize {
	const TYPE: DataPointType = DataPointType::I64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::I64(self as _)
	}
}

impl DataPointValue for f32 {
	const TYPE: DataPointType = DataPointType::F64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::F64(self as _)
	}
}

impl DataPointValue for f64 {
	const TYPE: DataPointType = DataPointType::F64;
	
	fn into(self) -> NumberDataPointValue {
		NumberDataPointValue::F64(self as _)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataPointType {
	I64,
	F64
}

pub struct BoundInstrument<T: DataPointValue> {
	inner: Instrument<T>,
	hash:  u64,
	attrs: Vec<(Cow<'static, str>, AnyValue)>
}

impl<T: DataPointValue> BoundInstrument<T> {
	pub fn record(&self, value: T) {
		let sync = self.inner.inner.rt.0.sync.lock().expect("failed to lock runtime");
		unsafe { &mut*self.inner.inner.data.get() }.record(self.hash, value.into());
		std::mem::drop(sync);
	}
}

#[derive(Clone)]
pub struct Instrument<T: DataPointValue> {
	inner:   Arc<InstrumentInner>,
	_marker: PhantomData<T>
}

impl<T: DataPointValue> Instrument<T> {
	pub(crate) fn new(runtime: &Runtime, params: InstrumentParameters) -> Self {
		let inner = Arc::new(InstrumentInner {
			name: params.name,
			unit: params.unit,
			desc: params.desc,
			data: UnsafeCell::new(InstrumentData::new(params.aggregation, T::TYPE)),
			call: None,
			rt:   runtime.clone()
		});
		
		runtime.0.sync.lock()
			.expect("failed to lock runtime")
			.instruments
			.insert(inner.clone());
		
		Instrument { inner, _marker: PhantomData }
	}
	
	pub(crate) fn new_observable(runtime: &Runtime, params: InstrumentParameters, callback: impl Fn(&Instrument<T>) + Send + Sync + 'static) {
		let instrument = Arc::new(InstrumentInner {
			name: params.name,
			unit: params.unit,
			desc: params.desc,
			data: UnsafeCell::new(InstrumentData::new(params.aggregation, T::TYPE)),
			call: Some(Box::new(move |v| {
				let instrument = Instrument {
					inner:   unsafe { Arc::from_raw(v) },
					_marker: PhantomData
				};
				(callback)(&instrument);
				std::mem::forget(instrument);
			})),
			rt:   runtime.clone()
		});
		
		runtime.0.sync.lock()
			.expect("failed to lock runtime")
			.instruments
			.insert(instrument);
	}
	
	pub fn bind(self, attributes: impl IntoIterator<Item = (Cow<'static, str>, AnyValue)>) -> BoundInstrument<T> {
		BoundInstrument { inner: self, hash: 0, attrs: attributes.into_iter().collect() }
	}
	
	pub fn record(&self, value: T, attributes: impl IntoIterator<Item = (Cow<'static, str>, AnyValue)>) {
		let sync = self.inner.rt.0.sync.lock().expect("failed to lock runtime");
		let hash = 0; // TODO attr hash
		unsafe { &mut*self.inner.data.get() }.record(hash, value.into());
		std::mem::drop(sync);
	}
}

impl<T: DataPointValue> std::fmt::Debug for Instrument<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.field("name", &self.inner.name)
			.field("unit", &self.inner.unit)
			.field("desc", &self.inner.desc)
			.field("data", &self.inner.data)
			.finish()
	}
}

pub(crate) struct InstrumentInner {
	name: Cow<'static, str>,
	unit: Option<Cow<'static, str>>,
	desc: Option<Cow<'static, str>>,
	data: UnsafeCell<InstrumentData>,
	call: Option<Box<dyn Fn(&Self) + Send + Sync + 'static>>,
	rt:   Runtime
}

unsafe impl Send for InstrumentInner {}
unsafe impl Sync for InstrumentInner {}

impl Hash for InstrumentInner {
	fn hash<H: Hasher>(&self, state: &mut H) {
		Borrow::<str>::borrow(&self.name).hash(state);
	}
}

impl PartialEq for InstrumentInner {
	fn eq(&self, other: &Self) -> bool {
		self.name.eq(&other.name)
	}
}

impl Eq for InstrumentInner {}

impl std::fmt::Debug for InstrumentInner {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>())
			.field("name", &self.name)
			.field("unit", &self.unit)
			.field("desc", &self.desc)
			.field("data", &self.data)
			.finish()
	}
}

pub enum InstrumentData {
	Drop,
	SumI64 {
		data_points: HashMap<u64, NumberDataPoint<i64>>,
		temporality: AggregationTemporality,
		monotonic:   bool
	},
	SumF64 {
		data_points: HashMap<u64, NumberDataPoint<f64>>,
		temporality: AggregationTemporality,
		monotonic:   bool
	},
	LastValueI64 {
		data_points: HashMap<u64, NumberDataPoint<i64>>,
	},
	LastValueF64 {
		data_points: HashMap<u64, NumberDataPoint<f64>>,
	},
	HistogramI64 {
		data_points: HashMap<u64, HistogramDataPoint<i64>>,
		temporality: AggregationTemporality
	},
	HistogramF64 {
		data_points: HashMap<u64, HistogramDataPoint<f64>>,
		temporality: AggregationTemporality
	}
}

impl InstrumentData {
	fn new(aggregation: Aggregation, ty: DataPointType) -> Self {
		match (aggregation, ty) {
			(Aggregation::LastValue, DataPointType::I64) => Self::LastValueI64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN)
			},
			(Aggregation::LastValue, DataPointType::F64) => Self::LastValueF64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN)
			},
			(Aggregation::Sum { temporality, monotonic }, DataPointType::I64) => Self::SumI64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN),
				temporality,
				monotonic
			},
			(Aggregation::Sum { temporality, monotonic }, DataPointType::F64) => Self::SumF64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN),
				temporality,
				monotonic
			},
			(Aggregation::Histogram { temporality }, DataPointType::I64) => Self::HistogramI64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN),
				temporality
			},
			(Aggregation::Histogram { temporality }, DataPointType::F64) => Self::HistogramF64 {
				data_points: HashMap::with_capacity(DEFAULT_BUF_LEN),
				temporality
			},
			(Aggregation::Drop, _) => Self::Drop
		}
	}
	
	fn record(&mut self, hash: u64, value: NumberDataPointValue) {
		match (self, value) {
			(Self::SumI64 { data_points, .. }, NumberDataPointValue::I64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.value += v;
			}
			(Self::SumF64 { data_points, .. }, NumberDataPointValue::F64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.value += v;
			}
			(Self::LastValueI64 { data_points, .. }, NumberDataPointValue::I64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.value = v;
			}
			(Self::LastValueF64 { data_points, .. }, NumberDataPointValue::F64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.value = v;
			}
			(Self::HistogramI64 { data_points, .. }, NumberDataPointValue::I64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.count += 1;
				data_point.sum += v;
			}
			(Self::HistogramF64 { data_points, .. }, NumberDataPointValue::F64(v)) => {
				let data_point = data_points.entry(hash).or_insert_with(Default::default);
				data_point.time = Instant::now();
				data_point.count += 1;
				data_point.sum += v;
			}
			(Self::Drop, _) => (),
			_ => unreachable!()
		}
	}
}

impl std::fmt::Debug for InstrumentData {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(match self {
			InstrumentData::Drop                => "Instrumantation::Drop",
			InstrumentData::SumI64 { .. }       => "Instrumantation::SumI64",
			InstrumentData::SumF64 { .. }       => "Instrumantation::SumF64",
			InstrumentData::LastValueI64 { .. } => "Instrumantation::LastValueI64",
			InstrumentData::LastValueF64 { .. } => "Instrumantation::LastValueF64",
			InstrumentData::HistogramI64 { .. } => "Instrumantation::HistogramI64",
			InstrumentData::HistogramF64 { .. } => "Instrumantation::HistogramF64"
		})
	}
}

#[derive(Clone, Debug)]
pub struct NumberDataPoint<T> {
	pub start_time: Instant,
	pub time:       Instant,
	pub value:      T,
	pub exemplars:  Vec<Exemplar>,
	pub flags:      u32
}

impl<T: Default> Default for NumberDataPoint<T> {
	fn default() -> Self {
		Self {
			start_time: Instant::now(),
			time:       Instant::now(),
			value:      T::default(),
			exemplars:  Vec::new(),
			flags:      0
		}
	}
}

pub struct HistogramDataPoint<T> {
	pub start_time:           Instant,
	pub time:                 Instant,
	pub count:                u64,
	pub sum:                  T,
	pub bucket_counts:        Vec<u64>,
	pub explicit_bounds:      Vec<T>,
	pub exemplars:            Vec<Exemplar>,
	pub flags:                u32
}

impl<T: Default> Default for HistogramDataPoint<T> {
	fn default() -> Self {
		Self {
			start_time:      Instant::now(),
			time:            Instant::now(),
			count:           0,
			sum:             T::default(),
			bucket_counts:   Vec::new(),
			explicit_bounds: Vec::new(),
			exemplars:       Vec::new(),
			flags:           0
		}
	}
}