#include "display_transition.h"
#include <assert.h>
#include <stdio.h>

int main(void)
{
    p4desk_display_transition_t s = {0};
    uint32_t elapsed = 999, duration = 999;
    assert(!p4dt_pending(&s));
    assert(!p4dt_arm(&s, 0));
    assert(!p4dt_arm(&s, 2001));
    assert(p4dt_arm(&s, 600));
    assert(p4dt_pending(&s));
    // Waiting for Mac creation cannot start the clock, nor can a stale JPEG.
    assert(!p4dt_sample(&s, 12, 4000000, &elapsed, &duration));
    p4dt_bind(&s, 12);
    assert(!p4dt_sample(&s, 11, 5000000, &elapsed, &duration));
    assert(p4dt_sample(&s, 12, 6000000, &elapsed, &duration));
    assert(elapsed == 0 && duration == 600);
    assert(p4dt_sample(&s, 12, 6300000, &elapsed, &duration));
    assert(elapsed == 300);
    assert(p4dt_sample(&s, 12, 7000000, &elapsed, &duration));
    assert(elapsed == 600 && p4dt_pending(&s));
    // Decoding the final frame is insufficient; require current LCD completion.
    p4dt_presented(&s, 11, true);
    p4dt_presented(&s, 12, false);
    assert(p4dt_pending(&s));
    p4dt_presented(&s, 12, true);
    assert(!p4dt_pending(&s));
    assert(!p4dt_sample(&s, 12, 7100000, &elapsed, &duration));
    assert(p4dt_arm(&s, 600));
    p4dt_bind(&s, 13);
    p4dt_cancel(&s);
    assert(!p4dt_sample(&s, 13, 8000000, &elapsed, &duration));
    // A new host session without an arm must never replay an old transition.
    assert(p4dt_arm(&s, 600));
    p4dt_bind(&s, 14);
    p4dt_bind(&s, 15);
    assert(!p4dt_pending(&s));
    for (uint32_t epoch = 16; epoch < 10016; ++epoch) {
        assert(p4dt_arm(&s, 600));
        p4dt_bind(&s, epoch);
        assert(p4dt_sample(&s, epoch, 0, &elapsed, &duration) && elapsed == 0);
        assert(p4dt_sample(&s, epoch, 600000, &elapsed, &duration) && elapsed == 600);
        p4dt_presented(&s, epoch, true);
        assert(!p4dt_pending(&s));
    }
    puts("display transition: delayed first JPEG, duration, DMA completion, cancel, epochs, 10000 cycles passed");
}
