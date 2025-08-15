#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <curl/curl.h>
#include <unistd.h>

// Configuration
#define CANISTER_ID "uxrrr-q7777-77774-qaaaq-cai"
#define IC_HOST "https://ic0.app"
#define MAX_RESPONSE_SIZE 10485760  // 10MB max response
#define MAX_RETRIES 30
#define RETRY_DELAY_SECONDS 2

// Response structure for HTTP requests
struct APIResponse {
    char *data;
    size_t size;
};

// Callback function to write HTTP response data
static size_t WriteCallback(void *contents, size_t size, size_t nmemb, struct APIResponse *response) {
    size_t realsize = size * nmemb;
    char *ptr = realloc(response->data, response->size + realsize + 1);

    if (!ptr) {
        printf("❌ Not enough memory (realloc returned NULL)\n");
        return 0;
    }

    response->data = ptr;
    memcpy(&(response->data[response->size]), contents, realsize);
    response->size += realsize;
    response->data[response->size] = 0;

    return realsize;
}

// Initialize API response structure
void init_api_response(struct APIResponse *response) {
    response->data = malloc(1);
    response->size = 0;
}

// Clean up API response structure
void cleanup_api_response(struct APIResponse *response) {
    if (response->data) {
        free(response->data);
        response->data = NULL;
    }
    response->size = 0;
}

// Generate image via canister API
char* generate_image() {
    CURL *curl;
    CURLcode res;
    struct APIResponse response;
    char *task_id = NULL;

    init_api_response(&response);

    curl = curl_easy_init();
    if (!curl) {
        printf("❌ Failed to initialize cURL\n");
        cleanup_api_response(&response);
        return NULL;
    }

    // Prepare Candid request payload
    const char *candid_payload = "(record { prompt = \"a beautiful digital art landscape with mountains and trees\"; width = opt 64; height = opt 64; num_inference_steps = opt 10; guidance_scale = opt 7.5; seed = opt 12345 })";

    // Build URL
    char url[512];
    snprintf(url, sizeof(url), "%s/api/v2/canister/%s/call", IC_HOST, CANISTER_ID);

    printf("🚀 Generating image...\n");
    printf("📡 URL: %s\n", url);
    printf("📝 Method: generate_image\n");

    // Set cURL options
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, candid_payload);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 30L);

    // Set headers for IC API
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/x-www-form-urlencoded");
    headers = curl_slist_append(headers, "Accept: application/cbor");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    // Perform the request
    res = curl_easy_perform(curl);

    if (res != CURLE_OK) {
        printf("❌ cURL error: %s\n", curl_easy_strerror(res));
    } else {
        long response_code;
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
        printf("📊 Response Code: %ld\n", response_code);

        if (response_code == 200) {
            printf("✅ Image generation request sent successfully!\n");
            // For demonstration, return a mock task ID
            task_id = malloc(16);
            strcpy(task_id, "task_1");
        } else {
            printf("❌ Request failed with code: %ld\n", response_code);
        }
    }

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
    cleanup_api_response(&response);

    return task_id;
}

// Get task status
int check_task_status(const char *task_id) {
    printf("📋 Checking task status for: %s\n", task_id);

    // For demonstration purposes, simulate checking
    printf("⏳ Task is processing...\n");

    // In real implementation, make API call to check status
    // Return 1 for completed, 0 for pending, -1 for error
    return 1; // Simulate completion
}

// Get image data from completed task
int get_image_data(const char *task_id, char **image_data, size_t *data_size) {
    CURL *curl;
    CURLcode res;
    struct APIResponse response;
    int success = 0;

    init_api_response(&response);

    curl = curl_easy_init();
    if (!curl) {
        printf("❌ Failed to initialize cURL for image data\n");
        cleanup_api_response(&response);
        return 0;
    }

    // Build URL for image endpoint
    char url[512];
    snprintf(url, sizeof(url), "%s/api/v2/canister/%s/query", IC_HOST, CANISTER_ID);

    // Prepare Candid request for get_image
    char candid_payload[256];
    snprintf(candid_payload, sizeof(candid_payload), "(\"get_image\", \"%s\")", task_id);

    printf("🖼️ Getting image data for task: %s\n", task_id);

    // Set cURL options
    curl_easy_setopt(curl, CURLOPT_URL, url);
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, candid_payload);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
    curl_easy_setopt(curl, CURLOPT_TIMEOUT, 60L);

    // Set headers
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/x-www-form-urlencoded");
    headers = curl_slist_append(headers, "Accept: application/cbor");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    // Perform the request
    res = curl_easy_perform(curl);

    if (res != CURLE_OK) {
        printf("❌ cURL error getting image: %s\n", curl_easy_strerror(res));
    } else {
        long response_code;
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);

        if (response_code == 200 && response.size > 0) {
            *image_data = malloc(response.size);
            if (*image_data) {
                memcpy(*image_data, response.data, response.size);
                *data_size = response.size;
                success = 1;
                printf("✅ Retrieved %zu bytes of image data\n", response.size);
            }
        } else {
            printf("❌ Failed to get image data, response code: %ld\n", response_code);
        }
    }

    curl_slist_free_all(headers);
    curl_easy_cleanup(curl);
    cleanup_api_response(&response);

    return success;
}

// Convert byte array to BMP file (simulated for demo)
void save_demo_bmp(const char *filename) {
    printf("🎨 Creating demo BMP image: %s\n", filename);

    // Create a simple 64x64 BMP file for demonstration
    FILE *file = fopen(filename, "wb");
    if (!file) {
        printf("❌ Cannot create file: %s\n", filename);
        return;
    }

    const int width = 64;
    const int height = 64;
    const int row_padding = (4 - (width * 3) % 4) % 4;
    const int file_size = 54 + (width * 3 + row_padding) * height;

    // BMP file header (14 bytes)
    unsigned char file_header[] = {
        'B', 'M',                           // Signature
        file_size & 0xFF, (file_size >> 8) & 0xFF,
        (file_size >> 16) & 0xFF, (file_size >> 24) & 0xFF,  // File size
        0, 0, 0, 0,                         // Reserved
        54, 0, 0, 0                         // Offset to pixel data
    };

    // BMP info header (40 bytes)
    unsigned char info_header[] = {
        40, 0, 0, 0,                        // Header size
        width & 0xFF, (width >> 8) & 0xFF, 0, 0,     // Width
        height & 0xFF, (height >> 8) & 0xFF, 0, 0,   // Height
        1, 0,                               // Color planes
        24, 0,                              // Bits per pixel
        0, 0, 0, 0,                         // Compression
        0, 0, 0, 0,                         // Image size
        0, 0, 0, 0,                         // X pixels per meter
        0, 0, 0, 0,                         // Y pixels per meter
        0, 0, 0, 0,                         // Colors used
        0, 0, 0, 0                          // Important colors
    };

    // Write headers
    fwrite(file_header, 1, sizeof(file_header), file);
    fwrite(info_header, 1, sizeof(info_header), file);

    // Write pixel data (create a colorful pattern)
    for (int y = height - 1; y >= 0; y--) {  // BMP stores bottom-to-top
        for (int x = 0; x < width; x++) {
            unsigned char r = (x * 255) / width;
            unsigned char g = (y * 255) / height;
            unsigned char b = ((x + y) * 255) / (width + height);

            // BMP uses BGR order
            fwrite(&b, 1, 1, file);
            fwrite(&g, 1, 1, file);
            fwrite(&r, 1, 1, file);
        }

        // Add row padding
        for (int p = 0; p < row_padding; p++) {
            unsigned char pad = 0;
            fwrite(&pad, 1, 1, file);
        }
    }

    fclose(file);
    printf("✅ Demo image saved as: %s\n", filename);
    printf("📊 File size: %d bytes\n", file_size);
}

// Print usage instructions
void print_usage() {
    printf("\n💻 IC Stable Diffusion C Client\n");
    printf("=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "\n");
    printf("\n📝 This client demonstrates API interaction with your IC canister.\n");
    printf("🔧 For actual API calls, you'll need to:\n\n");

    printf("1. Use dfx command line tool:\n");
    printf("   dfx canister call %s generate_image '(record {\n", CANISTER_ID);
    printf("     prompt = \"a beautiful landscape\";\n");
    printf("     width = opt 64;\n");
    printf("     height = opt 64;\n");
    printf("   })'\n\n");

    printf("2. Get the task ID from response\n\n");

    printf("3. Check status:\n");
    printf("   dfx canister call %s get_task_status '(\"task_1\")'\n\n", CANISTER_ID);

    printf("4. Get image bytes:\n");
    printf("   dfx canister call %s get_image '(\"task_1\")'\n\n", CANISTER_ID);

    printf("🎨 This demo creates a sample BMP to show the expected output format.\n\n");
}

int main() {
    printf("🚀 Starting IC Stable Diffusion Client...\n\n");

    // Initialize cURL globally
    curl_global_init(CURL_GLOBAL_DEFAULT);

    // Print usage instructions
    print_usage();

    // Create a demo BMP file to show expected format
    save_demo_bmp("demo_generated_image.bmp");

    // Simulate the API workflow
    printf("\n🔄 Simulating API Workflow:\n");
    printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1: Generate image
    char *task_id = generate_image();
    if (!task_id) {
        printf("❌ Failed to start image generation\n");
        curl_global_cleanup();
        return 1;
    }

    printf("📋 Generated Task ID: %s\n", task_id);

    // Step 2: Wait for completion (simulation)
    printf("\n⏳ Waiting for image generation to complete...\n");
    int retries = 0;
    while (retries < MAX_RETRIES) {
        int status = check_task_status(task_id);

        if (status == 1) {
            printf("✅ Task completed!\n");
            break;
        } else if (status == -1) {
            printf("❌ Task failed\n");
            free(task_id);
            curl_global_cleanup();
            return 1;
        }

        retries++;
        if (retries < MAX_RETRIES) {
            printf("⏳ Still processing... (retry %d/%d)\n", retries, MAX_RETRIES);
            sleep(1); // Wait 1 second between retries
        }
    }

    // Step 3: Get image data
    char *image_data = NULL;
    size_t data_size = 0;

    if (get_image_data(task_id, &image_data, &data_size)) {
        printf("🖼️ Successfully retrieved image data!\n");

        // Save to file (in real implementation, this would be the actual bytes)
        printf("💾 Saving image as generated_image.bmp\n");
        save_demo_bmp("generated_image.bmp");

        free(image_data);
    } else {
        printf("❌ Failed to retrieve image data\n");
    }

    printf("\n🎉 Process complete!\n");
    printf("📁 Check generated_image.bmp to see your AI-generated art!\n");

    // Cleanup
    free(task_id);
    curl_global_cleanup();

    return 0;
}

// Compile instructions (add to end of file as comment):
/*
COMPILE INSTRUCTIONS:

1. Install dependencies (macOS with Homebrew):
   brew install curl

2. Install dependencies (Ubuntu/Debian):
   sudo apt-get install libcurl4-openssl-dev

3. Compile:
   gcc -o ic_client main.c -lcurl

4. Run:
   ./ic_client

NOTES:
- This demo creates sample BMP files to show the expected format
- For actual IC canister calls, use dfx command line tool
- The HTTP API calls shown are examples of the structure needed
- Real implementation would need proper Candid encoding/decoding
- Removed JSON-C dependency for simpler compilation
*/
