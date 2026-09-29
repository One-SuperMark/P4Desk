// SPDX-License-Identifier: MIT
// Waveshare 7B schematic: BAT -> R92 200k -> GPIO20 -> R93 100k -> GND.
// ETA6098 STAT ("change") drives LED1 only, with no MCU connection.
#include "board_p4.h"
#include "esp_adc/adc_oneshot.h"
#include "esp_adc/adc_cali.h"
#include "esp_adc/adc_cali_scheme.h"
#include "esp_log.h"
#include "soc/adc_channel.h"

_Static_assert(ADC1_CHANNEL_4_GPIO_NUM == 20, "7B battery divider must use GPIO20");
static const char *TAG = "battery";
static adc_oneshot_unit_handle_t s_adc;
static adc_cali_handle_t s_calibration;

esp_err_t board_p4_battery_init(void)
{
    const adc_oneshot_unit_init_cfg_t unit = {.unit_id = ADC_UNIT_1};
    esp_err_t result = adc_oneshot_new_unit(&unit, &s_adc);
    if (result != ESP_OK) return result;
    const adc_oneshot_chan_cfg_t channel = {
        .atten = ADC_ATTEN_DB_12, .bitwidth = ADC_BITWIDTH_DEFAULT,
    };
    result = adc_oneshot_config_channel(s_adc, ADC_CHANNEL_4, &channel);
    if (result == ESP_OK) {
#if ADC_CALI_SCHEME_CURVE_FITTING_SUPPORTED
        const adc_cali_curve_fitting_config_t calibration = {
            .unit_id = ADC_UNIT_1, .chan = ADC_CHANNEL_4,
            .atten = ADC_ATTEN_DB_12, .bitwidth = ADC_BITWIDTH_DEFAULT,
        };
        result = adc_cali_create_scheme_curve_fitting(&calibration, &s_calibration);
#else
        result = ESP_ERR_NOT_SUPPORTED;
#endif
    }
    if (result != ESP_OK) {
        adc_oneshot_del_unit(s_adc);
        s_adc = NULL;
        return result; // Do not estimate voltage from an uncalibrated raw code.
    }
    ESP_LOGI(TAG, "GPIO20 calibrated voltage sampling ready, divider=3; charge status unavailable");
    return ESP_OK;
}

int32_t board_p4_battery_voltage_mv(void)
{
    if (!s_adc || !s_calibration) return -1;
    int samples[16];
    for (unsigned n = 0; n < 16; ++n) {
        if (adc_oneshot_read(s_adc, ADC_CHANNEL_4, &samples[n]) != ESP_OK) return -1;
        // Small fixed insertion sort; drop the two extremes at each end.
        for (unsigned j = n; j > 0 && samples[j] < samples[j - 1]; --j) {
            int temporary = samples[j]; samples[j] = samples[j - 1]; samples[j - 1] = temporary;
        }
    }
    int sum = 0, pin_mv = 0;
    for (unsigned n = 2; n < 14; ++n) sum += samples[n];
    if (adc_cali_raw_to_voltage(s_calibration, (sum + 6) / 12, &pin_mv) != ESP_OK) return -1;
    const int32_t battery_mv = pin_mv * 3;
    // Invalid/out-of-range voltage is unknown, never a fabricated percentage.
    return battery_mv >= 2500 && battery_mv <= 4500 ? battery_mv : -1;
}
