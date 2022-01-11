use serde::Deserialize;

use luminance_front::{
    blending::{BlendingMode, Blending, Equation, Factor},
    face_culling::FaceCulling,
    scissor::ScissorRegion,
};
use luminance_front::face_culling::{FaceCullingOrder, FaceCullingMode};
use luminance_front::render_state::RenderState;
use luminance_front::depth_stencil::{Comparison, StencilTest, StencilOp, StencilOperations, Write};

pub const RENDER_STATE_LOAD_ID: &str = "render_state";

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct RenderStateDef {
    /// Blending configuration.
    blending: Option<BlendingModeDef>,
    /// Depth test configuration.
    depth_test: Option<ComparisonDef>,
    /// Depth write configuration.
    depth_write: WriteDef,
    /// Stencil Test configuration
    stencil_test: Option<StencilTestDef>,
    /// Stencil Operations Configuration
    stencil_operations: StencilOperationsDef,
    /// Face culling configuration.
    face_culling: Option<FaceCullingDef>,
    /// Scissor region configuration.
    scissor: Option<ScissorRegionDef>,
}

impl From<RenderStateDef> for RenderState {
    fn from(rs: RenderStateDef) -> Self {
        let render_state = RenderState::default()
            .set_scissor(rs.scissor.and_then(|sr| Some(ScissorRegion::from(sr))))
            .set_depth_test(rs.depth_test.and_then(|dt| Some(Comparison::from(dt))))
            .set_depth_write(Write::from(rs.depth_write))
            .set_stencil_test(rs.stencil_test.and_then(|st| Some(StencilTest::from(st))))
            .set_stencil_operations(StencilOperations::from(rs.stencil_operations))
            .set_face_culling(rs.face_culling.and_then(|fc| Some(FaceCulling::from(fc))));

        match rs.blending {
            Some(BlendingModeDef::Combined(b)) => render_state.set_blending(b),
            Some(BlendingModeDef::Separate { rgb, alpha}) =>
                render_state.set_blending_separate(Blending::from(rgb), Blending::from(alpha)),
            _ => render_state
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum BlendingModeDef {
    Combined(BlendingDef),
    Separate {
        rgb: BlendingDef,
        alpha: BlendingDef,
    },
}

impl From<BlendingDef> for Option<Blending> {
    fn from(b: BlendingDef) -> Self {
        Some(Blending {
            equation: Equation::from(b.equation),
            src: Factor::from(b.src),
            dst: Factor::from(b.dst)
        })
    }
}

impl From<BlendingDef> for Blending {
    fn from(b: BlendingDef) -> Self {
        Blending {
            equation: Equation::from(b.equation),
            src: Factor::from(b.src),
            dst: Factor::from(b.dst)
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct BlendingDef {
    pub equation: EquationDef,
    pub src: FactorDef,
    pub dst: FactorDef,
}

#[derive(Deserialize, Clone, Debug)]
enum EquationDef {
    Additive,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

impl From<EquationDef> for Equation {
    fn from(eq: EquationDef) -> Self {
        match eq {
            EquationDef::Additive => Equation::Additive,
            EquationDef::Subtract => Equation::Subtract,
            EquationDef::ReverseSubtract => Equation::ReverseSubtract,
            EquationDef::Min => Equation::Min,
            EquationDef::Max => Equation::Max
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum FactorDef {
    One,
    Zero,
    SrcColor,
    SrcColorComplement,
    DestColor,
    DestColorComplement,
    SrcAlpha,
    SrcAlphaComplement,
    DstAlpha,
    DstAlphaComplement,
    SrcAlphaSaturate,
}

impl From<FactorDef> for Factor {
    fn from(f: FactorDef) -> Self {
        match f {
            FactorDef::One => Factor::One,
            FactorDef::Zero => Factor::Zero,
            FactorDef::SrcColor => Factor::SrcColor,
            FactorDef::SrcColorComplement => Factor::SrcColorComplement,
            FactorDef::DestColor => Factor::DestColor,
            FactorDef::DestColorComplement => Factor::DestColorComplement,
            FactorDef::SrcAlpha => Factor::SrcAlpha,
            FactorDef::SrcAlphaComplement => Factor::DstAlphaComplement,
            FactorDef::DstAlpha => Factor::DstAlpha,
            FactorDef::DstAlphaComplement => Factor::DstAlphaComplement,
            FactorDef::SrcAlphaSaturate => Factor::SrcAlphaSaturate
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum ComparisonDef {
    Never,
    Always,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl From<ComparisonDef> for Comparison {
    fn from(dc: ComparisonDef) -> Self {
        match dc {
            ComparisonDef::Never => Comparison::Never,
            ComparisonDef::Always => Comparison::Always,
            ComparisonDef::Equal => Comparison::Equal,
            ComparisonDef::NotEqual => Comparison::NotEqual,
            ComparisonDef::Less => Comparison::Less,
            ComparisonDef::LessOrEqual => Comparison::LessOrEqual,
            ComparisonDef::Greater => Comparison::Greater,
            ComparisonDef::GreaterOrEqual => Comparison::GreaterOrEqual
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum WriteDef {
    On,
    Off,
}

impl From<WriteDef> for Write {
    fn from(w: WriteDef) -> Self {
        match w {
            WriteDef::On => Write::On,
            WriteDef::Off => Write::Off
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct FaceCullingDef {
    pub order: FaceCullingOrderDef,
    pub mode: FaceCullingModeDef,
}

impl From<FaceCullingDef> for FaceCulling {
    fn from(fc: FaceCullingDef) -> Self {
        FaceCulling { order: FaceCullingOrder::from(fc.order), mode: FaceCullingMode::from(fc.mode) }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum FaceCullingOrderDef {
    CW,
    CCW,
}

impl From<FaceCullingOrderDef> for FaceCullingOrder {
    fn from(fco: FaceCullingOrderDef) -> Self {
        match fco {
            FaceCullingOrderDef::CW => FaceCullingOrder::CW,
            FaceCullingOrderDef::CCW => FaceCullingOrder::CCW
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum FaceCullingModeDef {
    Front,
    Back,
    Both,
}

impl From<FaceCullingModeDef> for FaceCullingMode {
    fn from(fcm: FaceCullingModeDef) -> Self {
        match fcm {
            FaceCullingModeDef::Front => FaceCullingMode::Front,
            FaceCullingModeDef::Back => FaceCullingMode::Back,
            FaceCullingModeDef::Both => FaceCullingMode::Both
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ScissorRegionDef {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl From<ScissorRegionDef> for ScissorRegion {
    fn from(sc: ScissorRegionDef) -> Self {
        ScissorRegion {
            x: sc.x,
            y: sc.y,
            width: sc.width,
            height: sc.height
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct StencilTestDef {
    comparison: ComparisonDef,
    reference: u8,
    mask: u8
}

impl From<StencilTestDef> for StencilTest {
    fn from(def: StencilTestDef) -> Self {
        Self {
            comparison: Comparison::from(def.comparison),
            reference: def.reference,
            mask: def.mask
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct StencilOperationsDef {
    depth_passes_stencil_fails: StencilOpDef,
    depth_fails_stencil_passes: StencilOpDef,
    depth_stencil_pass: StencilOpDef
}

impl From<StencilOperationsDef> for StencilOperations {
    fn from(def: StencilOperationsDef) -> Self {
        Self {
            depth_passes_stencil_fails: StencilOp::from(def.depth_passes_stencil_fails),
            depth_fails_stencil_passes: StencilOp::from(def.depth_fails_stencil_passes),
            depth_stencil_pass: StencilOp::from(def.depth_stencil_pass)
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
enum StencilOpDef {
    Keep,
    Zero,
    Replace,
    Increment,
    IncrementWrap,
    Decrement,
    DecrementWrap,
    Invert,
}

impl From<StencilOpDef> for StencilOp {
    fn from(def: StencilOpDef) -> Self {
        match def {
            StencilOpDef::Keep => StencilOp::Keep,
            StencilOpDef::Zero => StencilOp::Zero,
            StencilOpDef::Replace => StencilOp::Replace,
            StencilOpDef::Increment => StencilOp::Increment,
            StencilOpDef::IncrementWrap => StencilOp::IncrementWrap,
            StencilOpDef::Decrement => StencilOp::Decrement,
            StencilOpDef::DecrementWrap => StencilOp::DecrementWrap,
            StencilOpDef::Invert => StencilOp::Invert
        }
    }
}