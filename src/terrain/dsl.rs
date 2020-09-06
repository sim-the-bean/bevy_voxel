use std::{
    cmp::{Ordering, PartialEq, PartialOrd},
    fmt::{self, Display},
    ops::{Add, Div, Mul, Rem, Sub},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use rand::Rng;

use bevy::math::Vec3;

use crate::collections::lod_tree::Voxel;

use super::Chunk;

trait AsOption {
    fn as_option(self) -> Option<Value>;
}

impl AsOption for bool {
    fn as_option(self) -> Option<Value> {
        if self {
            Some(Value::Unit)
        } else {
            None
        }
    }
}

impl AsOption for Value {
    fn as_option(self) -> Option<Value> {
        Some(self)
    }
}

impl AsOption for Option<Value> {
    fn as_option(self) -> Option<Value> {
        self
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
    Float,
    Float3,
}

impl Type {
    pub fn default(&self) -> Value {
        match self {
            Self::Unit => Value::Unit,
            Self::Bool => Value::Bool(false),
            Self::Float => Value::Float(0.0),
            Self::Float3 => Value::Float3(Vec3::zero()),
        }
    }

    pub fn rand<R: Rng>(&self, rng: &mut R) -> Value {
        match self {
            Self::Unit => Value::Unit,
            Self::Bool => Value::Bool(rng.gen()),
            Self::Float => Value::Float(rng.gen()),
            Self::Float3 => Value::Float3(Vec3::new(rng.gen(), rng.gen(), rng.gen())),
        }
    }

    pub fn cast(&self, v: Value) -> Value {
        match self {
            Self::Unit => Value::Unit,
            Self::Bool => match v {
                Value::Unit => Value::Bool(false),
                Value::Bool(x) => Value::Bool(x),
                Value::Float(x) => Value::Bool(x >= 0.5),
                Value::Float3(x) => Value::Bool(x.length_squared() >= 1.0),
            },
            Self::Float => match v {
                Value::Unit => Value::Float(0.0),
                Value::Bool(false) => Value::Float(0.0),
                Value::Bool(true) => Value::Float(1.0),
                Value::Float(x) => Value::Float(x),
                Value::Float3(x) => Value::Float(x.length()),
            },
            Self::Float3 => match v {
                Value::Unit => Value::Float3(Vec3::zero()),
                Value::Bool(false) => Value::Float3(Vec3::zero()),
                Value::Bool(true) => Value::Float3(Vec3::new(1.0, 1.0, 1.0)),
                Value::Float(x) => Value::Float3(Vec3::new(x, x, x)),
                Value::Float3(x) => Value::Float3(x),
            },
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Bool => write!(f, "bool"),
            Self::Float => write!(f, "float"),
            Self::Float3 => write!(f, "float3"),
        }
    }
}

macro_rules! type_error {
    ($e:expr, $t:expr) => {{
        let e = $e;
        panic!("{}: {} is not of type {}", e, e.type_of(), $t)
    }};
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    Float(f32),
    Float3(Vec3),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Float(x) => write!(f, "{}", x),
            Self::Float3(x) => write!(f, "({}, {}, {})", x.x(), x.y(), x.z()),
        }
    }
}

impl Value {
    pub fn type_of(&self) -> Type {
        match self {
            Self::Unit => Type::Unit,
            Self::Bool(_) => Type::Bool,
            Self::Float(_) => Type::Float,
            Self::Float3(_) => Type::Float3,
        }
    }

    pub fn as_unit(&self) -> () {
        match self {
            Self::Unit => (),
            _ => type_error!(self, Type::Unit),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(x) => *x,
            _ => type_error!(self, Type::Bool),
        }
    }

    pub fn as_float(&self) -> f32 {
        match self {
            Self::Float(x) => *x,
            _ => type_error!(self, Type::Float),
        }
    }

    pub fn as_float3(&self) -> Vec3 {
        match self {
            Self::Float3(x) => *x,
            _ => type_error!(self, Type::Float3),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match self {
            Self::Float(this) => Self::Float(this + other.as_float()),
            Self::Float3(this) => Self::Float3(this + other.as_float3()),
            _ => type_error!(self, Type::Float),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match self {
            Self::Float(this) => Self::Float(this - other.as_float()),
            Self::Float3(this) => Self::Float3(this - other.as_float3()),
            _ => type_error!(self, Type::Float),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self {
            Self::Float(this) => Self::Float(this * other.as_float()),
            Self::Float3(this) => Self::Float3(this * other.as_float3()),
            _ => type_error!(self, Type::Float),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match self {
            Self::Float(this) => Self::Float(this / other.as_float()),
            Self::Float3(this) => Self::Float3(this / other.as_float3()),
            _ => type_error!(self, Type::Float),
        }
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match self {
            Self::Float(this) => Self::Float(this + other.as_float()),
            Self::Float3(this) => {
                let other = other.as_float3();
                Self::Float3(Vec3::new(
                    this.x() % other.x(),
                    this.y() % other.y(),
                    this.z() % other.z(),
                ))
            }
            _ => type_error!(self, Type::Float),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_float().partial_cmp(&other.as_float())
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unit,
    Bool(bool),
    Float(f32),
    Float3(Vec3),
    Rand(Type),
    Ratio(u32, u32),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Rem(Box<Expression>, Box<Expression>),
    Cast(Type, Box<Expression>),
}

impl Expression {
    pub fn execute<R: Rng>(&self, rng: &mut R) -> Value {
        match self {
            Self::Unit => Value::Unit,
            Self::Bool(x) => Value::Bool(*x),
            Self::Float(x) => Value::Float(*x),
            Self::Float3(x) => Value::Float3(*x),
            Self::Rand(t) => t.rand(rng),
            Self::Ratio(n, d) => Value::Bool(rng.gen_ratio(*n, *d)),
            Self::Add(a, b) => a.execute(rng) + b.execute(rng),
            Self::Sub(a, b) => a.execute(rng) - b.execute(rng),
            Self::Mul(a, b) => a.execute(rng) * b.execute(rng),
            Self::Div(a, b) => a.execute(rng) / b.execute(rng),
            Self::Rem(a, b) => a.execute(rng) % b.execute(rng),
            Self::Cast(t, e) => t.cast(e.execute(rng)),
        }
    }

    pub fn type_of(&self) -> Type {
        match self {
            Self::Unit => Type::Unit,
            Self::Bool(_) => Type::Bool,
            Self::Float(_) => Type::Float,
            Self::Float3(_) => Type::Float3,
            Self::Rand(t) => *t,
            Self::Cast(t, _) => *t,
            _ => todo!(),
        }
    }

    pub fn to_query(self) -> BlockQuery {
        BlockQuery::Expression(ExpressionQuery::ValueOf(self))
    }

    pub fn is_true(self) -> BlockQuery {
        BlockQuery::Expression(ExpressionQuery::IsTrue(self))
    }

    pub fn add(self, other: Self) -> Self {
        Self::Add(Box::new(self), Box::new(other))
    }

    pub fn sub(self, other: Self) -> Self {
        Self::Sub(Box::new(self), Box::new(other))
    }

    pub fn mul(self, other: Self) -> Self {
        Self::Mul(Box::new(self), Box::new(other))
    }

    pub fn div(self, other: Self) -> Self {
        Self::Div(Box::new(self), Box::new(other))
    }

    pub fn rem(self, other: Self) -> Self {
        Self::Rem(Box::new(self), Box::new(other))
    }

    pub fn cast(self, t: Type) -> Self {
        Self::Cast(t, Box::new(self))
    }
}

impl From<Value> for Expression {
    fn from(v: Value) -> Self {
        match v {
            Value::Unit => Self::Unit,
            Value::Bool(x) => Self::Bool(x),
            Value::Float(x) => Self::Float(x),
            Value::Float3(x) => Self::Float3(x),
        }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexQuery {
    Map(Box<BlockQuery>, Expression),
    Not(Box<BlockQuery>),
    And(Box<BlockQuery>, Box<BlockQuery>),
    Or(Box<BlockQuery>, Box<BlockQuery>),
}

impl ComplexQuery {
    pub fn execute<R: Rng, T: Voxel>(
        &self,
        rng: &mut R,
        xz: Option<(i32, i32)>,
        chunk: &Chunk<T>,
    ) -> Option<Value> {
        match self {
            ComplexQuery::Map(q, e) => q.execute(rng, xz, chunk).map(|_| e.execute(rng)),
            ComplexQuery::Not(q) => match q.execute(rng, xz, chunk) {
                Some(_) => None,
                None => Some(Value::Unit),
            },
            ComplexQuery::And(a, b) => a
                .execute(rng, xz, chunk)
                .and_then(|_| b.execute(rng, xz, chunk)),
            ComplexQuery::Or(a, b) => a
                .execute(rng, xz, chunk)
                .or_else(|| b.execute(rng, xz, chunk)),
        }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionQuery {
    ValueOf(Expression),
    IsTrue(Expression),
    TypeIs(Type, Expression),
    Eq(Expression, Expression),
    Ne(Expression, Expression),
    Lt(Expression, Expression),
    Gt(Expression, Expression),
    Le(Expression, Expression),
    Ge(Expression, Expression),
}

impl ExpressionQuery {
    pub fn execute<R: Rng>(&self, rng: &mut R) -> Option<Value> {
        match self {
            ExpressionQuery::ValueOf(e) => e.execute(rng).as_option(),
            ExpressionQuery::IsTrue(e) => e.execute(rng).as_bool().as_option(),
            ExpressionQuery::TypeIs(t, e) => (e.type_of() == *t).as_option(),
            ExpressionQuery::Eq(a, b) => (a.execute(rng) == b.execute(rng)).as_option(),
            ExpressionQuery::Ne(a, b) => (a.execute(rng) != b.execute(rng)).as_option(),
            ExpressionQuery::Lt(a, b) => (a.execute(rng) < b.execute(rng)).as_option(),
            ExpressionQuery::Gt(a, b) => (a.execute(rng) > b.execute(rng)).as_option(),
            ExpressionQuery::Le(a, b) => (a.execute(rng) <= b.execute(rng)).as_option(),
            ExpressionQuery::Ge(a, b) => (a.execute(rng) >= b.execute(rng)).as_option(),
        }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnQuery {
    YTop,
}

impl ColumnQuery {
    pub fn execute<T: Voxel>(&self, (x, z): (i32, i32), chunk: &Chunk<T>) -> Option<Value> {
        match self {
            ColumnQuery::YTop => {
                let h = chunk.width() as i32;
                if chunk.contains_key((x, h - 1, z)) {
                    return None;
                }
                for y in (0..chunk.width() as i32 - 1).rev() {
                    if chunk.contains_key((x, y, z)) {
                        return Some(Value::Float3(Vec3::new(x as _, y as f32 + 1.0, z as _)));
                    }
                }
                None
            }
        }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum BlockQuery {
    Complex(ComplexQuery),
    Expression(ExpressionQuery),
    Column(ColumnQuery),
}

impl BlockQuery {
    pub fn execute<R: Rng, T: Voxel>(
        &self,
        rng: &mut R,
        xz: Option<(i32, i32)>,
        chunk: &Chunk<T>,
    ) -> Option<Value> {
        match self {
            BlockQuery::Complex(q) => q.execute(rng, xz, chunk),
            BlockQuery::Expression(q) => q.execute(rng),
            BlockQuery::Column(q) => q.execute(
                xz.expect("column queries must be supplied with a xz coordinate"),
                chunk,
            ),
        }
    }

    pub fn y_top() -> Self {
        BlockQuery::Column(ColumnQuery::YTop)
    }

    pub fn and_then(self, other: Self) -> Self {
        BlockQuery::Complex(ComplexQuery::And(Box::new(self), Box::new(other)))
    }

    pub fn or_else(self, other: Self) -> Self {
        BlockQuery::Complex(ComplexQuery::Or(Box::new(self), Box::new(other)))
    }

    pub fn set_block<T: Voxel>(self, block: T) -> Statement<T> {
        Statement::SetBlock { q: self, block }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Statement<T: Voxel> {
    SetBlock {
        q: BlockQuery,
        block: T,
    },
    SetColumn {
        q: BlockQuery,
        h: BlockQuery,
        block: T,
    },
    Fill {
        p1: BlockQuery,
        p2: BlockQuery,
        block: T,
    },
}

impl<T: Voxel> Statement<T> {
    pub fn execute<R: Rng>(
        &self,
        rng: &mut R,
        xz: Option<(i32, i32)>,
        chunk: &Chunk<T>,
    ) -> Result<T> {
        let block = match self {
            Self::SetBlock { q, block } => q.execute(rng, xz, chunk).map(move |v| {
                let pos = v.as_float3();
                let (x, y, z) = (pos.x() as i32, pos.y() as i32, pos.z() as i32);
                BlockDiff {
                    at: (x, y, z),
                    size: (1, 1, 1),
                    data: vec![block.clone()],
                }
            }),
            _ => todo!(),
        };
        Result { block }
    }
}

#[derive(Debug, Clone)]
pub struct BlockDiff<T: Voxel> {
    pub(crate) at: (i32, i32, i32),
    pub(crate) size: (usize, usize, usize),
    pub(crate) data: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct Result<T: Voxel> {
    pub(crate) block: Option<BlockDiff<T>>,
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Octave {
    pub(crate) amplitude: f64,
    pub(crate) frequency: f64,
}

impl Octave {
    pub fn new(amplitude: f64, frequency: f64) -> Self {
        Self {
            amplitude,
            frequency,
        }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer<T: Voxel> {
    pub(crate) block: T,
    pub(crate) height: f64,
}

impl<T: Voxel> Layer<T> {
    pub fn new(block: T, height: f64) -> Self {
        Self { block, height }
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseType {
    Perlin,
    OpenSimplex,
    SuperSimplex,
}

impl Default for NoiseType {
    fn default() -> Self {
        Self::SuperSimplex
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseDimensions {
    Two,
    Three,
}

impl Default for NoiseDimensions {
    fn default() -> Self {
        Self::Two
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    NearestNeighbour,
    Bilinear(i32),
}

impl Filter {
    pub fn aux_width(&self) -> i32 {
        match self {
            Filter::NearestNeighbour => 0,
            Filter::Bilinear(_) => 1,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Filter::NearestNeighbour => 1,
            Filter::Bilinear(width) => *width,
        }
    }

    pub fn as_usize(&self) -> usize {
        self.as_i32() as _
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::Bilinear(2)
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Biome<T: Voxel> {
    pub(crate) name: Option<&'static str>,
    pub(crate) prob: f64,
    pub(crate) octaves: Vec<Octave>,
    pub(crate) layers: Vec<Layer<T>>,
    pub(crate) per_xz: Vec<Statement<T>>,
    pub(crate) per_chunk: Vec<Statement<T>>,
}

impl<T: Voxel> Default for Biome<T> {
    fn default() -> Self {
        Self {
            name: None,
            prob: 1.0,
            octaves: Vec::new(),
            layers: Vec::new(),
            per_xz: Vec::new(),
            per_chunk: Vec::new(),
        }
    }
}

impl<T: Voxel> Biome<T> {
    pub fn build() -> BiomeBuilder<T> {
        BiomeBuilder {
            inner: Self::default(),
        }
    }
}

pub struct BiomeBuilder<T: Voxel> {
    inner: Biome<T>,
}

impl<T: Voxel> BiomeBuilder<T> {
    pub fn build(self) -> Biome<T> {
        self.inner
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn spawn_probability(mut self, p: f64) -> Self {
        self.inner.prob = p;
        self
    }

    pub fn octave(mut self, o: Octave) -> Self {
        self.inner.octaves.push(o);
        self
    }

    pub fn layer(mut self, l: Layer<T>) -> Self {
        self.inner.layers.push(l);
        self
    }

    pub fn per_xz(mut self, s: Statement<T>) -> Self {
        self.inner.per_xz.push(s);
        self
    }

    pub fn per_chunk(mut self, s: Statement<T>) -> Self {
        self.inner.per_chunk.push(s);
        self
    }
}

#[cfg_attr(feature = "savedata", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Program<T: Voxel> {
    pub(crate) name: Option<&'static str>,
    pub(crate) seed: u32,
    pub(crate) chunk_size: u32,
    pub(crate) subdivisions: u32,
    pub(crate) filter: Filter,
    pub(crate) biome_frequency: f64,
    pub(crate) dimensions: NoiseDimensions,
    pub(crate) noise_type: NoiseType,
    pub(crate) biomes: Vec<Biome<T>>,
}

impl<T: Voxel> Default for Program<T> {
    fn default() -> Self {
        Self {
            name: None,
            seed: 0,
            chunk_size: 5,
            subdivisions: 0,
            filter: Default::default(),
            biome_frequency: 1.0,
            dimensions: Default::default(),
            noise_type: Default::default(),
            biomes: Vec::new(),
        }
    }
}

impl<T: Voxel> Program<T> {
    pub fn build() -> ProgramBuilder<T> {
        ProgramBuilder {
            inner: Self::default(),
        }
    }
}

pub struct ProgramBuilder<T: Voxel> {
    inner: Program<T>,
}

impl<T: Voxel> ProgramBuilder<T> {
    pub fn build(mut self) -> Program<T> {
        let sum = self
            .inner
            .biomes
            .iter()
            .map(|biome| biome.prob)
            .sum::<f64>();
        for biome in &mut self.inner.biomes {
            biome.prob /= sum;
        }
        self.inner
            .biomes
            .sort_unstable_by(|a, b| a.prob.partial_cmp(&b.prob).unwrap_or(Ordering::Equal));
        self.inner
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn biome_frequency(mut self, freq: f64) -> Self {
        self.inner.biome_frequency = freq;
        self
    }

    pub fn noise_dimensions(mut self, d: NoiseDimensions) -> Self {
        self.inner.dimensions = d;
        self
    }

    pub fn noise_type(mut self, n: NoiseType) -> Self {
        self.inner.noise_type = n;
        self
    }

    pub fn biome(mut self, b: Biome<T>) -> Self {
        self.inner.biomes.push(b);
        self
    }

    pub fn seed(mut self, seed: u32) -> Self {
        self.inner.seed = seed;
        self
    }

    pub fn chunk_size(mut self, size: u32) -> Self {
        self.inner.chunk_size = size;
        self
    }

    pub fn subdivisions(mut self, subdivisions: u32) -> Self {
        self.inner.subdivisions = subdivisions;
        self
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        match filter {
            Filter::NearestNeighbour => {}
            Filter::Bilinear(width) => assert!(
                (width as u32).is_power_of_two(),
                "bilinear filter must have a power of two width"
            ),
        }
        self.inner.filter = filter;
        self
    }
}
