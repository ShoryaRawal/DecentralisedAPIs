use candid::{CandidType, Decode, Encode, Nat};
use ic_cdk::api::management_canister::http_request::{HttpHeader, HttpResponse};
use ic_cdk::api::time;
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

// Memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdStore = StableBTreeMap<String, StorableGenerationTask, Memory>;

// HTTP Request structure for IC
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

// Data structures
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct GenerationTask {
    pub id: String,
    pub status: TaskStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
    pub request: GenerationRequest,
    pub result: Option<Vec<u8>>, // Base64 encoded image
    pub error: Option<String>,
}

// Storable wrapper for GenerationTask
#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct StorableGenerationTask(pub GenerationTask);

impl Storable for StorableGenerationTask {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

// Stable Diffusion Model Components
#[derive(Clone)]
pub struct StableDiffusionModel {
    pub tokenizer: SimpleTokenizer,
    pub text_encoder: TextEncoder,
    pub unet: UNet,
    pub vae_decoder: VAEDecoder,
    pub scheduler: DDIMScheduler,
}

#[derive(Clone)]
pub struct SimpleTokenizer {
    vocab_size: usize,
    max_length: usize,
}

#[derive(Clone)]
pub struct TextEncoder {
    embedding_dim: usize,
}

#[derive(Clone)]
pub struct UNet {
    in_channels: usize,
    out_channels: usize,
}

#[derive(Clone)]
pub struct VAEDecoder {
    latent_channels: usize,
}

#[derive(Clone)]
pub struct DDIMScheduler {
    num_train_timesteps: usize,
    beta_start: f32,
    beta_end: f32,
}

// Global state
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static TASK_STORE: RefCell<IdStore> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static MODEL: RefCell<Option<StableDiffusionModel>> = RefCell::new(None);

    static TASK_COUNTER: RefCell<u64> = RefCell::new(0);
}

impl SimpleTokenizer {
    fn new() -> Self {
        Self {
            vocab_size: 49408, // CLIP tokenizer vocab size
            max_length: 77,    // CLIP max sequence length
        }
    }

    fn encode(&self, text: &str) -> Vec<u32> {
        // Simplified tokenization - in real implementation, use proper CLIP tokenizer
        let mut tokens = vec![49406]; // Start token

        // Simple word splitting and hashing for demo
        for word in text.split_whitespace().take(self.max_length - 2) {
            let token_id =
                (word.chars().map(|c| c as u32).sum::<u32>() % (self.vocab_size as u32 - 2)) + 1;
            tokens.push(token_id);
        }

        tokens.push(49407); // End token

        // Pad to max_length
        while tokens.len() < self.max_length {
            tokens.push(0);
        }

        tokens.truncate(self.max_length);
        tokens
    }
}

impl TextEncoder {
    fn new() -> Self {
        Self { embedding_dim: 768 }
    }

    fn encode(&self, tokens: &[u32]) -> Vec<f32> {
        // Simplified text encoding - returns mock embeddings
        // In real implementation, this would use actual CLIP text encoder weights
        let mut embeddings = Vec::with_capacity(tokens.len() * self.embedding_dim);

        for &token in tokens {
            for i in 0..self.embedding_dim {
                let value = (token as f32 + i as f32) / 1000.0;
                embeddings.push(value.sin()); // Simple deterministic "embedding"
            }
        }

        embeddings
    }
}

impl UNet {
    fn new() -> Self {
        Self {
            in_channels: 4,
            out_channels: 4,
        }
    }

    fn forward(&self, latents: &[f32], timestep: u32, text_embeddings: &[f32]) -> Vec<f32> {
        // Simplified UNet forward pass
        // In real implementation, this would be the actual diffusion model
        let mut noise_pred = latents.to_vec();

        // Simple noise prediction based on timestep and text conditioning
        let conditioning_strength =
            (text_embeddings.iter().sum::<f32>() / text_embeddings.len() as f32) * 0.1;
        let time_factor = (timestep as f32 / 1000.0).cos();

        for (i, pred) in noise_pred.iter_mut().enumerate() {
            *pred += conditioning_strength * time_factor * ((i as f32).sin() * 0.1);
        }

        noise_pred
    }
}

impl VAEDecoder {
    fn new() -> Self {
        Self { latent_channels: 4 }
    }

    fn decode(&self, latents: &[f32]) -> Vec<u8> {
        let width = 64u32; // Smaller size for demo
        let height = 64u32;

        // Generate a pattern based on latents that looks more like generated art
        let latent_sum = latents.iter().sum::<f32>() / latents.len() as f32;
        let latent_variance = latents
            .iter()
            .map(|&x| (x - latent_sum).powi(2))
            .sum::<f32>()
            / latents.len() as f32;

        // Create a simple bitmap (BMP format for simplicity)
        self.create_bmp(width, height, latents, latent_sum, latent_variance)
    }

    fn create_bmp(
        &self,
        width: u32,
        height: u32,
        latents: &[f32],
        avg: f32,
        variance: f32,
    ) -> Vec<u8> {
        let mut bmp_data = Vec::new();

        // BMP file header (14 bytes)
        bmp_data.extend_from_slice(b"BM"); // Signature
        let file_size = 54 + (width * height * 3); // Header + pixel data
        bmp_data.extend_from_slice(&file_size.to_le_bytes());
        bmp_data.extend_from_slice(&[0, 0, 0, 0]); // Reserved
        bmp_data.extend_from_slice(&54u32.to_le_bytes()); // Offset to pixel data

        // DIB header (40 bytes)
        bmp_data.extend_from_slice(&40u32.to_le_bytes()); // Header size
        bmp_data.extend_from_slice(&width.to_le_bytes());
        bmp_data.extend_from_slice(&height.to_le_bytes());
        bmp_data.extend_from_slice(&1u16.to_le_bytes()); // Color planes
        bmp_data.extend_from_slice(&24u16.to_le_bytes()); // Bits per pixel
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Compression
        bmp_data.extend_from_slice(&(width * height * 3).to_le_bytes()); // Image size
        bmp_data.extend_from_slice(&2835u32.to_le_bytes()); // X pixels per meter
        bmp_data.extend_from_slice(&2835u32.to_le_bytes()); // Y pixels per meter
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Colors used
        bmp_data.extend_from_slice(&0u32.to_le_bytes()); // Important colors

        // Generate pixel data (BGR format, bottom-to-top)
        let row_padding = (4 - (width * 3) % 4) % 4;

        for y in (0..height).rev() {
            // BMP stores rows bottom-to-top
            for x in 0..width {
                // Create interesting patterns based on latents
                let latent_idx = ((x + y * width) as usize) % latents.len();
                let latent_val = latents[latent_idx];

                // Generate colors based on position and latent values
                let norm_x = x as f32 / width as f32;
                let norm_y = y as f32 / height as f32;

                let r = ((norm_x * 255.0) + (latent_val * 50.0) + (avg * 100.0)).clamp(0.0, 255.0)
                    as u8;
                let g = ((norm_y * 255.0) + (variance * 200.0) + (latent_val * 30.0))
                    .clamp(0.0, 255.0) as u8;
                let b = (((norm_x + norm_y) * 127.5) + (latent_val * 70.0)).clamp(0.0, 255.0) as u8;

                // BMP uses BGR order
                bmp_data.push(b);
                bmp_data.push(g);
                bmp_data.push(r);
            }

            // Add row padding
            for _ in 0..row_padding {
                bmp_data.push(0);
            }
        }

        bmp_data
    }
}

impl DDIMScheduler {
    fn new() -> Self {
        Self {
            num_train_timesteps: 1000,
            beta_start: 0.00085,
            beta_end: 0.012,
        }
    }

    fn step(&self, noise_pred: &[f32], timestep: u32, latents: &[f32]) -> Vec<f32> {
        // Simplified DDIM sampling step
        let alpha = 1.0
            - self.beta_start
            - (self.beta_end - self.beta_start)
                * (timestep as f32 / self.num_train_timesteps as f32);
        let beta = 1.0 - alpha;

        latents
            .iter()
            .zip(noise_pred.iter())
            .map(|(&latent, &noise)| latent - beta.sqrt() * noise)
            .collect()
    }

    fn get_timesteps(&self, num_inference_steps: usize) -> Vec<u32> {
        let step_size = self.num_train_timesteps / num_inference_steps;
        (0..num_inference_steps)
            .map(|i| (self.num_train_timesteps - i * step_size - 1) as u32)
            .collect()
    }
}

impl StableDiffusionModel {
    fn new() -> Self {
        Self {
            tokenizer: SimpleTokenizer::new(),
            text_encoder: TextEncoder::new(),
            unet: UNet::new(),
            vae_decoder: VAEDecoder::new(),
            scheduler: DDIMScheduler::new(),
        }
    }

    fn generate_image(&self, request: &GenerationRequest) -> Result<Vec<u8>, String> {
        // Set defaults
        let width = request.width.unwrap_or(512);
        let height = request.height.unwrap_or(512);
        let num_steps = request.num_inference_steps.unwrap_or(20);
        let guidance_scale = request.guidance_scale.unwrap_or(7.5);
        let seed = request.seed.unwrap_or(42);

        // Tokenize and encode text
        let tokens = self.tokenizer.encode(&request.prompt);
        let text_embeddings = self.text_encoder.encode(&tokens);

        // Handle negative prompt
        let negative_embeddings = if let Some(ref neg_prompt) = request.negative_prompt {
            let neg_tokens = self.tokenizer.encode(neg_prompt);
            self.text_encoder.encode(&neg_tokens)
        } else {
            let empty_tokens = self.tokenizer.encode("");
            self.text_encoder.encode(&empty_tokens)
        };

        // Initialize random latents
        let latent_size = (width / 8) * (height / 8) * 4; // VAE downsampling factor of 8
        let mut latents = self.generate_random_latents(latent_size as usize, seed);

        // Diffusion process
        let timesteps = self.scheduler.get_timesteps(num_steps as usize);

        for &timestep in &timesteps {
            // Predict noise with positive prompt
            let noise_pred_pos = self.unet.forward(&latents, timestep, &text_embeddings);

            // Predict noise with negative prompt
            let noise_pred_neg = self.unet.forward(&latents, timestep, &negative_embeddings);

            // Apply classifier-free guidance
            let noise_pred: Vec<f32> = noise_pred_neg
                .iter()
                .zip(noise_pred_pos.iter())
                .map(|(&neg, &pos)| neg + guidance_scale * (pos - neg))
                .collect();

            // Scheduler step
            latents = self.scheduler.step(&noise_pred, timestep, &latents);
        }

        // Decode latents to image
        let image_bytes = self.vae_decoder.decode(&latents);

        Ok(image_bytes)
    }

    fn generate_random_latents(&self, size: usize, seed: u64) -> Vec<f32> {
        // Simple pseudo-random number generation
        let mut latents = Vec::with_capacity(size);
        let mut rng_state = seed;

        for _ in 0..size {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let random_val = (rng_state as f32 / u64::MAX as f32) * 2.0 - 1.0;
            latents.push(random_val * 0.18215); // SD latent scaling factor
        }

        latents
    }
}

// Helper functions
fn generate_task_id() -> String {
    TASK_COUNTER.with(|counter| {
        let mut c = counter.borrow_mut();
        *c += 1;
        format!("task_{}", *c)
    })
}

fn get_current_time() -> u64 {
    time()
}

// API Endpoints

#[update]
async fn generate_image(request: GenerationRequest) -> ApiResponse<String> {
    let task_id = generate_task_id();
    let current_time = get_current_time();

    // Create initial task
    let mut task = GenerationTask {
        id: task_id.clone(),
        status: TaskStatus::Pending,
        created_at: current_time,
        completed_at: None,
        request: request.clone(),
        result: None,
        error: None,
    };

    // Process the image generation first
    let result = MODEL.with(|model| {
        if let Some(ref sd_model) = *model.borrow() {
            sd_model.generate_image(&request)
        } else {
            Err("Model not initialized".to_string())
        }
    });

    // Update task with result before storing
    match result {
        Ok(image_bytes) => {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(get_current_time());
            task.result = Some(image_bytes);
        }
        Err(error_msg) => {
            task.status = TaskStatus::Failed;
            task.completed_at = Some(get_current_time());
            task.error = Some(error_msg);
        }
    }

    // Store the completed task in a single borrow
    TASK_STORE.with(|store| {
        store
            .borrow_mut()
            .insert(task_id.clone(), StorableGenerationTask(task));
    });

    ApiResponse {
        success: true,
        data: Some(task_id),
        error: None,
        timestamp: current_time,
    }
}

#[query]
fn get_task_status(task_id: String) -> ApiResponse<GenerationTask> {
    TASK_STORE.with(|store| {
        if let Some(StorableGenerationTask(task)) = store.borrow().get(&task_id) {
            ApiResponse {
                success: true,
                data: Some(task),
                error: None,
                timestamp: get_current_time(),
            }
        } else {
            ApiResponse {
                success: false,
                data: None,
                error: Some("Task not found".to_string()),
                timestamp: get_current_time(),
            }
        }
    })
}

#[query]
fn get_image(task_id: String) -> ApiResponse<Vec<u8>> {
    TASK_STORE.with(|store| {
        if let Some(StorableGenerationTask(task)) = store.borrow().get(&task_id) {
            if let Some(ref image_data) = task.result {
                ApiResponse {
                    success: true,
                    data: Some(image_data.clone()),
                    error: None,
                    timestamp: get_current_time(),
                }
            } else {
                ApiResponse {
                    success: false,
                    data: None,
                    error: Some("Image not ready or generation failed".to_string()),
                    timestamp: get_current_time(),
                }
            }
        } else {
            ApiResponse {
                success: false,
                data: None,
                error: Some("Task not found".to_string()),
                timestamp: get_current_time(),
            }
        }
    })
}

#[query]
fn list_tasks() -> ApiResponse<Vec<String>> {
    TASK_STORE.with(|store| {
        let task_ids: Vec<String> = store.borrow().iter().map(|(id, _)| id).collect();

        ApiResponse {
            success: true,
            data: Some(task_ids),
            error: None,
            timestamp: get_current_time(),
        }
    })
}

// HTTP Interface for external access
#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    let path = req.url.trim_start_matches('/');

    match (req.method.as_str(), path) {
        ("GET", path) if path.starts_with("task/") => {
            let task_id = path.strip_prefix("task/").unwrap_or("");
            let response = get_task_status(task_id.to_string());

            HttpResponse {
                status: Nat::from(if response.success { 200u16 } else { 404u16 }),
                headers: vec![HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                }],
                body: serde_json::to_string(&response)
                    .unwrap_or_default()
                    .into_bytes(),
            }
        }
        ("GET", path) if path.starts_with("image/") => {
            let task_id = path.strip_prefix("image/").unwrap_or("");
            let response = get_image(task_id.to_string());

            if response.success && response.data.is_some() {
                HttpResponse {
                    status: Nat::from(200u16),
                    headers: vec![HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "image/bmp".to_string(),
                    }],
                    body: response.data.unwrap(),
                }
            } else {
                HttpResponse {
                    status: Nat::from(404u16),
                    headers: vec![HttpHeader {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                    }],
                    body: serde_json::to_string(&response)
                        .unwrap_or_default()
                        .into_bytes(),
                }
            }
        }
        ("GET", "tasks") => {
            let response = list_tasks();
            HttpResponse {
                status: Nat::from(200u16),
                headers: vec![HttpHeader {
                    name: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                }],
                body: serde_json::to_string(&response)
                    .unwrap_or_default()
                    .into_bytes(),
            }
        }
        ("POST", "generate") => {
            // Parse JSON body
            let body_str = String::from_utf8_lossy(&req.body);
            match serde_json::from_str::<GenerationRequest>(&body_str) {
                Ok(_request) => {
                    // Note: This is a query function, so we can't actually call the update function
                    // In a real implementation, you'd need to handle this differently
                    let error_response = ApiResponse::<String> {
                        success: false,
                        data: None,
                        error: Some(
                            "Use the canister's generate_image method directly".to_string(),
                        ),
                        timestamp: get_current_time(),
                    };

                    HttpResponse {
                        status: Nat::from(400u16),
                        headers: vec![HttpHeader {
                            name: "Content-Type".to_string(),
                            value: "application/json".to_string(),
                        }],
                        body: serde_json::to_string(&error_response)
                            .unwrap_or_default()
                            .into_bytes(),
                    }
                }
                Err(_) => {
                    let error_response = ApiResponse::<String> {
                        success: false,
                        data: None,
                        error: Some("Invalid JSON request".to_string()),
                        timestamp: get_current_time(),
                    };

                    HttpResponse {
                        status: Nat::from(400u16),
                        headers: vec![HttpHeader {
                            name: "Content-Type".to_string(),
                            value: "application/json".to_string(),
                        }],
                        body: serde_json::to_string(&error_response)
                            .unwrap_or_default()
                            .into_bytes(),
                    }
                }
            }
        }
        _ => HttpResponse {
            status: Nat::from(404u16),
            headers: vec![HttpHeader {
                name: "Content-Type".to_string(),
                value: "text/plain".to_string(),
            }],
            body: b"Not Found".to_vec(),
        },
    }
}

// Lifecycle hooks
#[init]
fn init() {
    // Initialize the Stable Diffusion model
    MODEL.with(|model| {
        *model.borrow_mut() = Some(StableDiffusionModel::new());
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    // State is automatically preserved in stable structures
}

#[post_upgrade]
fn post_upgrade() {
    // Reinitialize the model after upgrade
    init();
}
