#include <stdlib.h>
#include <string.h>
#include "board_p4.h"
#include "p4desk_hal.h"
#include "p4desk_runtime.h"
#include "esp_check.h"
#include "esp_heap_caps.h"
#include "esp_log.h"
#include "esp_partition.h"
#include "esp_spiffs.h"

static const char *TAG = "p4desk";

static bool partition_is_blank(const esp_partition_t *partition)
{
    uint8_t *buffer = heap_caps_malloc(4096, MALLOC_CAP_INTERNAL | MALLOC_CAP_8BIT);
    if (!buffer) return false;
    bool blank = true;
    for (size_t offset = 0; offset < partition->size && blank; offset += 4096) {
        size_t count = partition->size - offset;
        if (count > 4096) count = 4096;
        if (esp_partition_read(partition, offset, buffer, count) != ESP_OK) {
            blank = false;
            break;
        }
        for (size_t n = 0; n < count; n++) {
            if (buffer[n] != 0xff) { blank = false; break; }
        }
    }
    free(buffer);
    return blank;
}

static void flash_mount(void)
{
    const esp_vfs_spiffs_conf_t config = {
        .base_path = "/flash", .partition_label = "storage", .max_files = 8,
        .format_if_mount_failed = false,
    };
    esp_err_t err = esp_vfs_spiffs_register(&config);
    if (err == ESP_OK) return;
    const esp_partition_t *partition = esp_partition_find_first(
        ESP_PARTITION_TYPE_DATA, ESP_PARTITION_SUBTYPE_DATA_SPIFFS, "storage");
    // Only a genuinely empty new partition may be initialized. Existing data is
    // preserved on a mount failure; neither corrupted flash nor TF is formatted.
    if (partition && partition_is_blank(partition)) {
        ESP_LOGI(TAG, "initializing blank settings partition");
        err = esp_spiffs_format("storage");
        if (err == ESP_OK) err = esp_vfs_spiffs_register(&config);
    }
    if (err != ESP_OK) ESP_LOGW(TAG, "settings storage unavailable; data preserved: %s", esp_err_to_name(err));
}

void app_main(void)
{
    static board_p4_t board;
    ESP_LOGI(TAG, "P4Desk starting, protocol v1");
    ESP_ERROR_CHECK(board_p4_init(&board));
    flash_mount();
    p4desk_runtime_init(&board);
    p4desk_usb_init();
    ESP_LOGI(TAG, "Rust UI starting");
    rust_main_entry();
    ESP_LOGE(TAG, "Rust UI returned unexpectedly");
    abort();
}
