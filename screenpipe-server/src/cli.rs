use std::{path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand, ValueHint};
use clap_complete::{generate, Shell};
use clap::CommandFactory;
use screenpipe_audio::{vad::{VadSensitivity, VadEngineEnum}, core::engine::AudioTranscriptionEngine as CoreAudioTranscriptionEngine};
use screenpipe_vision::{custom_ocr::CustomOcrConfig, utils::OcrEngine as CoreOcrEngine};
use clap::ValueEnum;
use screenpipe_core::Language;
use screenpipe_db::OcrEngine as DBOcrEngine;
use screenpipe_db::CustomOcrConfig as DBCustomOcrConfig;
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum CliAudioTranscriptionEngine {
    #[clap(name = "deepgram")]
    Deepgram,
    #[clap(name = "whisper-tiny")]
    WhisperTiny,
    #[clap(name = "whisper-tiny-quantized")]
    WhisperTinyQuantized,
    #[clap(name = "whisper-large")]
    WhisperLargeV3,
    #[clap(name = "whisper-large-quantized")]
    WhisperLargeV3Quantized,
    #[clap(name = "whisper-large-v3-turbo")]
    WhisperLargeV3Turbo,
    #[clap(name = "whisper-large-v3-turbo-quantized")]
    WhisperLargeV3TurboQuantized,
}

impl From<CliAudioTranscriptionEngine> for CoreAudioTranscriptionEngine {
    fn from(cli_engine: CliAudioTranscriptionEngine) -> Self {
        match cli_engine {
            CliAudioTranscriptionEngine::Deepgram => CoreAudioTranscriptionEngine::Deepgram,
            CliAudioTranscriptionEngine::WhisperTiny => CoreAudioTranscriptionEngine::WhisperTiny,
            CliAudioTranscriptionEngine::WhisperTinyQuantized => CoreAudioTranscriptionEngine::WhisperTinyQuantized,
            CliAudioTranscriptionEngine::WhisperLargeV3 => CoreAudioTranscriptionEngine::WhisperLargeV3,
            CliAudioTranscriptionEngine::WhisperLargeV3Quantized => CoreAudioTranscriptionEngine::WhisperLargeV3Quantized,
            CliAudioTranscriptionEngine::WhisperLargeV3Turbo => {
                CoreAudioTranscriptionEngine::WhisperLargeV3Turbo
            }
            CliAudioTranscriptionEngine::WhisperLargeV3TurboQuantized => {
                CoreAudioTranscriptionEngine::WhisperLargeV3TurboQuantized
            }
        }
    }
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum CliOcrEngine {
    Unstructured,
    #[cfg(target_os = "linux")]
    Tesseract,
    #[cfg(target_os = "windows")]
    WindowsNative,
    #[cfg(target_os = "macos")]
    AppleNative,
    Custom,
}

impl From<CliOcrEngine> for Arc<DBOcrEngine> {
    fn from(cli_engine: CliOcrEngine) -> Self {
        match cli_engine {
            CliOcrEngine::Unstructured => Arc::new(DBOcrEngine::Unstructured),
            #[cfg(target_os = "macos")]
            CliOcrEngine::AppleNative => Arc::new(DBOcrEngine::AppleNative),
            #[cfg(target_os = "linux")]
            CliOcrEngine::Tesseract => Arc::new(DBOcrEngine::Tesseract),
            #[cfg(target_os = "windows")]
            CliOcrEngine::WindowsNative => Arc::new(DBOcrEngine::WindowsNative),
            CliOcrEngine::Custom => Arc::new(DBOcrEngine::Custom(DBCustomOcrConfig::default())),
        }
    }
}

impl From<CliOcrEngine> for CoreOcrEngine {
    fn from(cli_engine: CliOcrEngine) -> Self {
        match cli_engine {
            CliOcrEngine::Unstructured => CoreOcrEngine::Unstructured,
            #[cfg(target_os = "linux")]
            CliOcrEngine::Tesseract => CoreOcrEngine::Tesseract,
            #[cfg(target_os = "windows")]
            CliOcrEngine::WindowsNative => CoreOcrEngine::WindowsNative,
            #[cfg(target_os = "macos")]
            CliOcrEngine::AppleNative => CoreOcrEngine::AppleNative,
            CliOcrEngine::Custom => {
                // Try to read config from environment variable
                if let Ok(config_str) = std::env::var("SCREENPIPE_CUSTOM_OCR_CONFIG") {
                    match serde_json::from_str(&config_str) {
                        Ok(config) => CoreOcrEngine::Custom(config),
                        Err(e) => {
                            tracing::warn!("failed to parse custom ocr config from env: {}", e);
                            CoreOcrEngine::Custom(CustomOcrConfig::default())
                        }
                    }
                } else {
                    CoreOcrEngine::Custom(CustomOcrConfig::default())
                }
            }
        }
    }
}
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum CliVadEngine {
    #[clap(name = "webrtc")]
    WebRtc,
    #[clap(name = "silero")]
    Silero,
}

impl From<CliVadEngine> for VadEngineEnum {
    fn from(cli_engine: CliVadEngine) -> Self {
        match cli_engine {
            CliVadEngine::WebRtc => VadEngineEnum::WebRtc,
            CliVadEngine::Silero => VadEngineEnum::Silero,
        }
    }
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum CliVadSensitivity {
    Low,
    Medium,
    High,
}

impl From<CliVadSensitivity> for VadSensitivity {
    fn from(cli_sensitivity: CliVadSensitivity) -> Self {
        match cli_sensitivity {
            CliVadSensitivity::Low => VadSensitivity::Low,
            CliVadSensitivity::Medium => VadSensitivity::Medium,
            CliVadSensitivity::High => VadSensitivity::High,
        }
    }
}

#[derive(Parser)]
#[command(
    author, 
    version,
    about, 
    long_about = None,
    name = "screenpipe"
)]
pub struct Cli {
    /// FPS for continuous recording
    /// 1 FPS = 30 GB / month
    /// 5 FPS = 150 GB / month
    /// Optimise based on your needs.
    /// Your screen rarely change more than 1 times within a second, right?
    #[cfg_attr(not(target_os = "macos"), arg(short, long, default_value_t = 1.0))]
    #[cfg_attr(target_os = "macos", arg(short, long, default_value_t = 0.5))] 
    pub fps: f64, // ! not crazy about this (inconsistent behaviour across platforms) see https://github.com/mediar-ai/screenpipe/issues/173
    
    /// Audio chunk duration in seconds
    #[arg(short = 'd', long, default_value_t = 30)]
    pub audio_chunk_duration: u64,

    /// Port to run the server on
    #[arg(short = 'p', long, default_value_t = 3030)]
    pub port: u16,

    /// Disable audio recording
    #[arg(long, default_value_t = false)]
    pub disable_audio: bool,

    /// Audio devices to use (can be specified multiple times)
    #[arg(short = 'i', long)]
    pub audio_device: Vec<String>,

    // Audio devices to use for realtime audio transcription
    #[arg(short = 'r', long)]
    pub realtime_audio_device: Vec<String>,

    /// Data directory. Default to $HOME/.screenpipe
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub data_dir: Option<String>,

    /// Enable debug logging for screenpipe modules
    #[arg(long)]
    pub debug: bool,

    /// Audio transcription engine to use.
    /// Deepgram is a very high quality cloud-based transcription service (free of charge on us for now), recommended for high quality audio.
    /// WhisperTiny is a local, lightweight transcription model, recommended for high data privacy.
    /// WhisperDistilLargeV3 is a local, lightweight transcription model (-a whisper-large), recommended for higher quality audio than tiny.
    /// WhisperLargeV3Turbo is a local, lightweight transcription model (-a whisper-large-v3-turbo), recommended for higher quality audio than tiny.
    #[arg(short = 'a', long, value_enum, default_value_t = CliAudioTranscriptionEngine::WhisperTinyQuantized)]
    pub audio_transcription_engine: CliAudioTranscriptionEngine,

    /// Enable realtime audio transcription
    #[arg(long, default_value_t = false)]
    pub enable_realtime_audio_transcription: bool,

    /// Enable realtime vision
    #[arg(long, default_value_t = true)]
    pub enable_realtime_vision: bool,

    /// OCR engine to use.
    /// AppleNative is the default local OCR engine for macOS.
    /// WindowsNative is a local OCR engine for Windows.
    /// Unstructured is a cloud OCR engine (free of charge on us for now), recommended for high quality OCR.
    /// Tesseract is a local OCR engine (not supported on macOS)
    #[cfg_attr(
        target_os = "macos",
        arg(short = 'o', long, value_enum, default_value_t = CliOcrEngine::AppleNative)
    )]
    #[cfg_attr(
        target_os = "windows",
        arg(short = 'o', long, value_enum, default_value_t = CliOcrEngine::WindowsNative)
    )]
    #[cfg_attr(
        not(any(target_os = "macos", target_os = "windows")),
        arg(short = 'o', long, value_enum, default_value_t = CliOcrEngine::Tesseract)
    )]
    pub ocr_engine: CliOcrEngine,

    /// Monitor IDs to use, these will be used to select the monitors to record
    #[arg(short = 'm', long)]
    pub monitor_id: Vec<u32>,

    #[arg(short = 'l', long, value_enum)]
    pub language: Vec<Language>,

    /// Enable PII removal from OCR text property that is saved to db and returned in search results
    #[arg(long, default_value_t = false)]
    pub use_pii_removal: bool,

    /// Disable vision recording
    #[arg(long, default_value_t = false)]
    pub disable_vision: bool,

    /// VAD engine to use for speech detection
    #[arg(long, value_enum, default_value_t = CliVadEngine::Silero)] // Silero or WebRtc
    pub vad_engine: CliVadEngine,

    /// List of windows to ignore (by title) for screen recording - we use contains to match, example:
    /// --ignored-windows "Spotify" --ignored-windows "Bit" will ignore both "Bitwarden" and "Bittorrent"
    /// --ignored-windows "x" will ignore "Home / X" and "SpaceX"
    #[arg(long)]
    pub ignored_windows: Vec<String>,

    /// List of windows to include (by title) for screen recording - we use contains to match, example:
    /// --included-windows "Chrome" will include "Google Chrome"
    /// --included-windows "WhatsApp" will include "WhatsApp"
    #[arg(long)]
    pub included_windows: Vec<String>,

    /// Video chunk duration in seconds
    #[arg(long, default_value_t = 60)]
    pub video_chunk_duration: u64,

    /// Deepgram API Key for audio transcription
    #[arg(long = "deepgram-api-key")]
    pub deepgram_api_key: Option<String>,

    /// PID to watch for auto-destruction. If provided, screenpipe will stop when this PID is no longer running.
    #[arg(long)]
    pub auto_destruct_pid: Option<u32>,

    /// Voice activity detection sensitivity level
    #[arg(long, value_enum, default_value_t = CliVadSensitivity::High)]
    pub vad_sensitivity: CliVadSensitivity,

    /// Disable telemetry
    #[arg(long, default_value_t = false)]
    pub disable_telemetry: bool,

    /// Enable Local LLM API
    #[arg(long, default_value_t = false)]
    pub enable_llm: bool,

    /// Enable UI monitoring (macOS only)
    #[arg(long, default_value_t = false)]
    pub enable_ui_monitoring: bool,
    
    /// Enable experimental video frame cache (may increase CPU usage) - makes timeline UI available, frame streaming, etc.
    #[arg(long, default_value_t = true)]
    pub enable_frame_cache: bool,

    /// Capture windows that are not focused (default: false)
    #[arg(long, default_value_t = false)]
    pub capture_unfocused_windows: bool,

    /// Enable pipe functionality (default: false)
    #[arg(long, default_value_t = false)]
    pub enable_pipe_manager: bool,

    #[command(subcommand)]
    pub command: Option<Command>,

}



impl Cli {
    pub fn unique_languages(&self) -> Result<Vec<Language>, String> {
        let mut unique_langs = std::collections::HashSet::new();
        for lang in &self.language {
            if !unique_langs.insert(lang.clone()) {
                // continue don't care
            }
        }
        Ok(unique_langs.into_iter().collect())
    }
    pub fn handle_completions(&self, shell: Shell) -> anyhow::Result<()> {
        let mut cmd = Self::command();
        generate(shell, &mut cmd, "screenpipe", &mut std::io::stdout());
        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Audio device management commands
    Audio {
        #[command(subcommand)]
        subcommand: AudioCommand,
    },
    /// Vision device management commands
    Vision {
        #[command(subcommand)]
        subcommand: VisionCommand,
    },
    /// Pipe management commands
    Pipe {
        #[command(subcommand)]
        subcommand: PipeCommand,
    },
    /// MCP Server management commands
    Mcp {
        #[command(subcommand)]
        subcommand: McpCommand,
    },
    /// Add video files to existing screenpipe data (OCR only) - DOES NOT SUPPORT AUDIO
    Add {
        /// Path to folder containing video files
        path: String,
        /// Data directory. Default to $HOME/.screenpipe
        #[arg(long, value_hint = ValueHint::DirPath)]
        data_dir: Option<String>,
        /// Output format
        #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Regex pattern to filter files (e.g. "monitor.*\.mp4$")
        #[arg(long)]
        pattern: Option<String>,
        /// OCR engine to use
        #[arg(short = 'o', long, value_enum)]
        ocr_engine: Option<CliOcrEngine>,
        /// Path to JSON file containing metadata overrides
        #[arg(long, value_hint = ValueHint::FilePath)]
        metadata_override: Option<PathBuf>,
        /// Copy videos to screenpipe data directory
        #[arg(long, default_value_t = true)]
        copy_videos: bool,
        /// Enable debug logging for screenpipe modules
        #[arg(long)]
        debug: bool,
        /// Enable embedding generation for OCR text
        #[arg(long, default_value_t = false)]
        use_embedding: bool,
    },
    /// Run data migrations in the background
    Migrate {
        /// The name of the migration to run
        #[arg(long, default_value = "ocr_text_to_frames")]
        migration_name: String,
        /// Data directory. Default to $HOME/.screenpipe
        #[arg(long, value_hint = ValueHint::DirPath)]
        data_dir: Option<String>,
        /// The subcommand for data migration
        #[command(subcommand)]
        subcommand: Option<MigrationSubCommand>,
        /// Output format
        #[arg(short = 'o', long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Batch size for processing records
        #[arg(long, default_value_t = 100_000)]
        batch_size: i64,
        /// Delay between batches in milliseconds
        #[arg(long, default_value_t = 100)]
        batch_delay_ms: u64,
        /// Continue processing if errors occur
        #[arg(long, default_value_t = true)]
        continue_on_error: bool,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum MigrationSubCommand {
    /// Start or resume a migration
    Start,
    /// Pause a running migration
    Pause,
    /// Stop a running migration
    Stop,
    /// Get migration status
    Status,
}

#[derive(Subcommand)]
pub enum AudioCommand {
    /// List available audio devices
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum VisionCommand {
    /// List available monitors and vision devices
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(Subcommand)]
pub enum PipeCommand {
    /// List all pipes
    List {
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Download a new pipe (deprecated: use 'install' instead)
    #[deprecated(since = "0.2.26", note = "please use `install` instead")]
    Download {
        /// URL of the pipe to download
        url: String,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Install a new pipe
    Install {
        /// URL of the pipe to install
        url: String,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Get info for a specific pipe
    Info {
        /// ID of the pipe
        id: String,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Enable a pipe
    Enable {
        /// ID of the pipe to enable
        id: String,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Disable a pipe
    Disable {
        /// ID of the pipe to disable
        id: String,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Update pipe configuration
    Update {
        /// ID of the pipe to update
        id: String,
        /// New configuration as a JSON string
        config: String,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Purge all pipes
    Purge {
        /// Automatically confirm purge without prompting
        #[arg(short = 'y', long)]
        yes: bool,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
    /// Delete a pipe
    Delete {
        /// ID of the pipe to delete
        id: String,
        /// Automatically confirm deletion without prompting
        #[arg(short = 'y', long)]
        yes: bool,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum McpCommand {
    /// Setup MCP server configuration
    Setup {
        /// Directory to save MCP files (default: $HOME/.screenpipe/mcp)
        #[arg(long, value_hint = ValueHint::DirPath)]
        directory: Option<String>,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
        /// Server port
        #[arg(short = 'p', long, default_value_t = 3030)]
        port: u16,
        /// Force update existing files
        #[arg(long)]
        update: bool,
        /// Purge existing MCP directory before setup
        #[arg(long)]
        purge: bool,
    },
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}
