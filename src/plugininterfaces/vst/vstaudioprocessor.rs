use std::ffi::CStr;

/// Data-less struct acting as namespace
///
/// Holds CStrs for component types used as subcategories in PClassInfo2
pub struct PlugTypeName;
impl PlugTypeName {
    const kFx: &'static CStr = c"Fx";
    const kFxAnalyzer: &'static CStr = c"Fx|Analyzer";
    const kFxBass: &'static CStr = c"Fx|Bass";
    const kFxChannelStrip: &'static CStr = c"Fx|Channel Strip";
    const kFxDelay: &'static CStr = c"Fx|Delay";
    const kFxDistortion: &'static CStr = c"Fx|Distortion";
    const kFxDrums: &'static CStr = c"Fx|Drums";
    const kFxDynamics: &'static CStr = c"Fx|Dynamics";
    const kFxEQ: &'static CStr = c"Fx|Eq";
    const kFxFilter: &'static CStr = c"Fx|Filter";
    const kFxGenerator: &'static CStr = c"Fx|Generator";
    const kFxGuitar: &'static CStr = c"Fx|Guitar";
    const kFxInstrument: &'static CStr = c"Fx|Instrument";
    const kFxInstrumentExternal: &'static CStr = c"Fx|Instrument|External";
    const kFxMastering: &'static CStr = c"Fx|Mastering";
    const kFxMicrophone: &'static CStr = c"Fx|Microphone";
    const kFxModulation: &'static CStr = c"Fx|Modulation";
    const kFxNetwork: &'static CStr = c"Fx|Network";
    const kFxPitchShift: &'static CStr = c"Fx|Pitch Shift";
    const kFxRestoration: &'static CStr = c"Fx|Restoration";
    const kFxReverb: &'static CStr = c"Fx|Reverb";
    const kFxSpatial: &'static CStr = c"Fx|Spatial";
    const kFxSurround: &'static CStr = c"Fx|Surround";
    const kFxTools: &'static CStr = c"Fx|Tools";
    const kFxVocals: &'static CStr = c"Fx|Vocals";

    const kInstrument: &'static CStr = c"Instrument";
    const kInstrumentDrum: &'static CStr = c"Instrument|Drum";
    const kInstrumentExternal: &'static CStr = c"Instrument|External";
    const kInstrumentPiano: &'static CStr = c"Instrument|Piano";
    const kInstrumentSampler: &'static CStr = c"Instrument|Sampler";
    const kInstrumentSynth: &'static CStr = c"Instrument|Synth";
    const kInstrumentSynthSampler: &'static CStr = c"Instrument|Synth|Sampler";

    const kAmbisonics: &'static CStr = c"Ambisonics";
    const kAnalyzer: &'static CStr = c"Analyzer";
    const kNoOfflineProcess: &'static CStr = c"NoOfflineProcess";
    const kOnlyARA: &'static CStr = c"OnlyARA";
    const kOnlyOfflineProcess: &'static CStr = c"OnlyOfflineProcess";
    const kOnlyRealTime: &'static CStr = c"OnlyRT";
    const kSpatial: &'static CStr = c"Spatial";
    const kSpatialFx: &'static CStr = c"Spatial|Fx";
    const kUpDownMix: &'static CStr = c"Up-Downmix";

    const kMono: &'static CStr = c"Mono";
    const kStereo: &'static CStr = c"Stereo";
    const kSurround: &'static CStr = c"Surround";
}

#[allow(non_camel_case_types)]
pub enum kFx {
    Analyzer,
    Bass,
    ChannelStrip,
    Delay,
    Distortion,
    Drums,
    Dynamics,
    EQ,
    Filter,
    Generator,
    Guitar,
    Instrument,
    InstrumentExternal,
    Mastering,
    Microphone,
    Modulation,
    Network,
    PitchShift,
    Restoration,
    Reverb,
    Spatial,
    Surround,
    Tools,
    Vocals,
}
impl kFx {
    pub fn as_cstr(&self) -> &'static CStr {
        match self {
            kFx::Analyzer => PlugTypeName::kFx,
            kFx::Bass => PlugTypeName::kFxAnalyzer,
            kFx::ChannelStrip => PlugTypeName::kFxChannelStrip,
            kFx::Delay => PlugTypeName::kFxDelay,
            kFx::Distortion => PlugTypeName::kFxDistortion,
            kFx::Drums => PlugTypeName::kFxDrums,
            kFx::Dynamics => PlugTypeName::kFxDynamics,
            _ => todo!(),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum PlugType {
    kFx,
}
impl PlugType {
    pub fn as_cstr(&self) -> &'static CStr {
        match self {
            PlugType::kFx => PlugTypeName::kFx,
        }
    }
}
