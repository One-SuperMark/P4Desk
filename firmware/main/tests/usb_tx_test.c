#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "usb_tx_queue.h"
#include "touch_exit_notice.h"

typedef struct {
    p4desk_tx_packet_t packets[16];
    size_t count, capacity;
    bool consume_on_full;
    p4desk_tx_packet_t consumer_owned;
} fake_queue_t;

static struct { void *pointer; bool alive; } allocations[128];
static unsigned allocated, disposed;

static p4desk_tx_packet_t packet(uint8_t id)
{
    assert(allocated < 128);
    uint8_t *bytes = malloc(1);
    assert(bytes);
    *bytes = id;
    allocations[allocated].pointer = bytes;
    allocations[allocated++].alive = true;
    return (p4desk_tx_packet_t){.bytes = bytes, .length = 1, .generation = 7};
}

static void dispose(void *bytes)
{
    bool found = false;
    for (unsigned n = 0; n < allocated; n++) {
        if (allocations[n].pointer == bytes && allocations[n].alive) {
            allocations[n].alive = false;
            found = true;
            break;
        }
    }
    assert(found); // Every pointer is disposed exactly once by its owner.
    disposed++;
    free(bytes);
}

static bool receive(void *context, p4desk_tx_packet_t *out)
{
    fake_queue_t *queue = context;
    if (!queue->count) return false;
    *out = queue->packets[0];
    queue->count--;
    memmove(queue->packets, queue->packets + 1, queue->count * sizeof(*out));
    return true;
}

static bool send(void *context, const p4desk_tx_packet_t *in)
{
    fake_queue_t *queue = context;
    if (queue->count == queue->capacity) {
        // Model the consumer winning a receive after the producer observes a
        // full mailbox. Its packet must not be freed by the replacing producer.
        if (queue->consume_on_full) {
            queue->consume_on_full = false;
            assert(receive(queue, &queue->consumer_owned));
        }
        return false;
    }
    queue->packets[queue->count++] = *in;
    return true;
}

static p4desk_tx_queue_t adapter(fake_queue_t *queue)
{
    return (p4desk_tx_queue_t){.context = queue, .send = send, .receive = receive, .dispose = dispose};
}

typedef struct { unsigned attempts, failures; } notice_sender_t;
static bool send_notice(void *context)
{
    notice_sender_t *sender = context;
    sender->attempts++;
    if (sender->failures) { sender->failures--; return false; }
    return true;
}

int main(void)
{
    fake_queue_t mailbox = {.capacity = 1}, controls = {.capacity = 12}, events = {.capacity = 4};
    p4desk_tx_queue_t release = adapter(&mailbox), priority = adapter(&controls), normal = adapter(&events);
    for (unsigned n = 0; n < 12; n++) assert(p4desk_tx_queue_put(&priority, packet(10 + n), false));
    for (unsigned n = 0; n < 4; n++) assert(p4desk_tx_queue_put(&normal, packet(30 + n), true));
    assert(!p4desk_tx_queue_put(&priority, packet(49), false));
    assert(disposed == 1);

    // ACK saturation cannot reject a touch release. Replacing its pending
    // predecessor disposes that predecessor, never the in-flight message.
    assert(p4desk_tx_queue_put(&release, packet(50), true));
    assert(p4desk_tx_queue_put(&release, packet(51), true));
    assert(disposed == 2 && mailbox.count == 1 && controls.count == 12);
    p4desk_tx_packet_t active = packet(99);
    assert(p4desk_tx_select_next(&release, &priority, &normal, 7, &active));
    assert(*active.bytes == 99 && mailbox.count == 1); // Never insert into active TX.
    dispose(active.bytes);
    active = (p4desk_tx_packet_t){0};
    assert(p4desk_tx_select_next(&release, &priority, &normal, 7, &active));
    assert(*active.bytes == 51 && mailbox.count == 0 && controls.count == 12);
    dispose(active.bytes);
    active = (p4desk_tx_packet_t){0};
    assert(p4desk_tx_select_next(&release, &priority, &normal, 7, &active));
    assert(*active.bytes == 10 && controls.count == 11 && events.count == 4);
    dispose(active.bytes);
    active = (p4desk_tx_packet_t){0};
    p4desk_tx_queue_clear(&release);
    p4desk_tx_queue_clear(&priority);
    p4desk_tx_queue_clear(&normal);
    assert(allocated == disposed);
    assert(!p4desk_tx_select_next(&release, &priority, &normal, 7, &active));

    // Simulate the important producer/consumer ownership interleaving.
    assert(p4desk_tx_queue_put(&release, packet(60), true));
    mailbox.consume_on_full = true;
    unsigned before = disposed;
    assert(p4desk_tx_queue_put(&release, packet(61), true));
    assert(disposed == before && *mailbox.consumer_owned.bytes == 60);
    dispose(mailbox.consumer_owned.bytes);
    p4desk_tx_queue_clear(&release);
    assert(allocated == disposed);

    // Clear all three queues as on disconnect/reconnect: pending release also
    // relinquishes its payload, with no stale pointer retained by the mailbox.
    assert(p4desk_tx_queue_put(&release, packet(70), true));
    assert(p4desk_tx_queue_put(&priority, packet(71), false));
    assert(p4desk_tx_queue_put(&normal, packet(72), true));
    p4desk_tx_queue_clear(&release);
    p4desk_tx_queue_clear(&priority);
    p4desk_tx_queue_clear(&normal);
    assert(allocated == disposed);

    // A paused producer may enqueue after the old connection was cleared.
    // Generation rejection frees the late packet before selecting fresh data.
    p4desk_tx_packet_t stale = packet(80);
    stale.generation = 6;
    assert(p4desk_tx_queue_put(&release, stale, true));
    assert(p4desk_tx_queue_put(&priority, packet(81), false));
    active = (p4desk_tx_packet_t){0};
    before = disposed;
    assert(p4desk_tx_select_next(&release, &priority, &normal, 7, &active));
    assert(disposed == before + 1 && *active.bytes == 81 && mailbox.count == 0);
    dispose(active.bytes);
    assert(allocated == disposed);

    p4desk_exit_notice_t notice = {0};
    notice_sender_t sender = {.failures = 2};
    p4desk_exit_notice_begin(&notice, 8, 7);
    for (unsigned n = 0; n < 2; n++) {
        p4desk_exit_notice_service(&notice, true, true, 8, 7, send_notice, &sender);
        assert(notice.pending);
    }
    p4desk_exit_notice_service(&notice, true, true, 8, 7, send_notice, &sender);
    assert(!notice.pending && sender.attempts == 3);
    p4desk_exit_notice_service(&notice, true, true, 8, 7, send_notice, &sender);
    assert(sender.attempts == 3);
    // A new session, mode, disconnect or reconnect cancels stale retries.
    for (unsigned n = 0; n < 4; n++) {
        p4desk_exit_notice_begin(&notice, 8, 7);
        p4desk_exit_notice_service(&notice, n != 0, n != 1, n == 2 ? 9 : 8,
                                   n == 3 ? 9 : 7, send_notice, &sender);
        assert(!notice.pending && sender.attempts == 3);
    }
    puts("USB TX ownership, release mailbox and exit retry tests passed");
    return 0;
}
