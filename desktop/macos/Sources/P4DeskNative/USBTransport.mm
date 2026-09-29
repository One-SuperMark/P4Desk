#import <Foundation/Foundation.h>
#import <IOKit/IOKitLib.h>
#import <IOKit/IOKitKeys.h>
#import <IOKit/IOMessage.h>
#import <IOKit/usb/USB.h>
#import <IOUSBHost/IOUSBHost.h>
#include <mach/mach_time.h>
#import "P4DeskNative.h"

// Device Speed and USBSpeed use different SDK enums. Neither is an endpoint
// descriptor capability: both properties describe the current device link.
static uint32_t P4DeviceSpeedMbps(uint32_t speed) {
    switch (speed) {
        case kUSBDeviceSpeedFull: return 12;
        case kUSBDeviceSpeedHigh: return 480;
        case kUSBDeviceSpeedSuper: return 5000;
        case kUSBDeviceSpeedSuperPlus: return 10000;
        default: return 0;
    }
}
static uint32_t P4HostConnectionSpeedMbps(uint32_t speed) {
    switch (speed) {
        case kIOUSBHostConnectionSpeedFull: return 12;
        case kIOUSBHostConnectionSpeedHigh: return 480;
        case kIOUSBHostConnectionSpeedSuper: return 5000;
        case kIOUSBHostConnectionSpeedSuperPlus: return 10000;
        default: return 0;
    }
}
static bool P4RegistryUInt32(io_registry_entry_t entry, CFStringRef key, uint32_t *value) {
    CFTypeRef property = IORegistryEntryCreateCFProperty(entry, key, kCFAllocatorDefault, 0);
    if (!property) return false;
    int64_t number = 0;
    bool valid = CFGetTypeID(property) == CFNumberGetTypeID() &&
        CFNumberGetValue((CFNumberRef)property, kCFNumberSInt64Type, &number) &&
        number >= 0 && number <= UINT32_MAX;
    CFRelease(property);
    if (valid) *value = (uint32_t)number;
    return valid;
}
static uint32_t P4NegotiatedSpeedMbps(io_registry_entry_t interfaceService) {
    io_registry_entry_t entry = interfaceService;
    IOObjectRetain(entry);
    for (unsigned depth = 0; entry && depth < 32; depth++) {
        if (IOObjectConformsTo(entry, "IOUSBHostDevice")) {
            uint32_t speed = 0, mbps = 0;
            if (P4RegistryUInt32(entry, CFSTR(kUSBDevicePropertySpeed), &speed)) {
                mbps = P4DeviceSpeedMbps(speed);
            } else if (P4RegistryUInt32(entry, CFSTR(kUSBHostMatchingPropertySpeed), &speed)) {
                mbps = P4HostConnectionSpeedMbps(speed);
            }
            // Stop at the interface's device, even if its speed is unavailable.
            // An upstream hub can have a different negotiated link speed.
            IOObjectRelease(entry);
            return mbps;
        }
        io_registry_entry_t parent = 0;
        kern_return_t result = IORegistryEntryGetParentEntry(entry, kIOServicePlane, &parent);
        IOObjectRelease(entry);
        entry = result == KERN_SUCCESS ? parent : 0;
    }
    if (entry) IOObjectRelease(entry);
    return 0;
}
static uint64_t P4MonotonicNanoseconds(void) {
    static const mach_timebase_info_data_t timebase = [] {
        mach_timebase_info_data_t result;
        mach_timebase_info(&result);
        return result;
    }();
    return (uint64_t)(((__uint128_t)mach_absolute_time() * timebase.numer) / timebase.denom);
}

@interface P4Transfer : NSObject
@property(nonatomic, strong) NSData *data;
@property(nonatomic) NSUInteger offset;
@property(nonatomic) uint32_t token;
@property(nonatomic) BOOL video;
@property(nonatomic) uint64_t outStartedNanoseconds;
@end
@implementation P4Transfer
@end

@interface P4USBSession : NSObject
@property(nonatomic) P4USBCallback callback;
@property(nonatomic) void *context;
@property(nonatomic) uint16_t vendorID;
@property(nonatomic) uint16_t productID;
@property(nonatomic, strong) dispatch_queue_t queue;
@property(nonatomic, strong) dispatch_source_t timer;
@property(nonatomic, strong) IOUSBHostInterface *interface;
@property(nonatomic, strong) IOUSBHostPipe *input;
@property(nonatomic, strong) IOUSBHostPipe *output;
@property(nonatomic, strong) NSMutableArray<P4Transfer *> *controls;
@property(nonatomic, strong) P4Transfer *latestVideo;
@property(nonatomic, strong) P4Transfer *inFlight;
@property(nonatomic) BOOL running;
@property(nonatomic) uint64_t epoch;
@property(nonatomic) BOOL reportedOpenError;
@property(nonatomic) BOOL sending;
@property(nonatomic) NSTimeInterval connectAfter;
- (void)scan;
- (void)disconnect:(const char *)reason;
- (void)readNext;
- (void)pump;
@end

@implementation P4USBSession
- (void)emit:(uint32_t)event token:(uint32_t)token data:(NSData *)data reason:(const char *)reason {
    if (self.running && self.callback) self.callback(self.context, event, token,
        data ? (const uint8_t *)data.bytes : nullptr, data.length, reason);
}
- (void)scan {
    if (!self.running || self.interface || NSProcessInfo.processInfo.systemUptime < self.connectAfter) return;
    CFMutableDictionaryRef matching = IOServiceMatching("IOUSBHostInterface");
    if (!matching) return;
    // macOS 27's helper puts these properties at the dictionary's top level,
    // which matches no interfaces on the verified host. IOPropertyMatch applies
    // them to the IOUSBHostInterface service itself and preserves exact filtering.
    NSDictionary *properties = @{@"idVendor": @(self.vendorID), @"idProduct": @(self.productID),
        @"bInterfaceNumber": @0, @"bInterfaceClass": @255};
    CFDictionarySetValue(matching, CFSTR(kIOPropertyMatchKey), (__bridge CFDictionaryRef)properties);
    io_iterator_t iterator = 0;
    if (IOServiceGetMatchingServices(kIOMainPortDefault, matching, &iterator) != KERN_SUCCESS) return;
    io_service_t service;
    while ((service = IOIteratorNext(iterator))) {
        NSError *error = nil;
        __weak P4USBSession *weakSelf = self;
        IOUSBHostInterface *interface = [[IOUSBHostInterface alloc] initWithIOService:service
            options:IOUSBHostObjectInitOptionsNone queue:self.queue error:&error
            interestHandler:^(IOUSBHostObject *object, uint32_t message, void *argument) {
                if (message == kIOMessageServiceIsTerminated) {
                    P4USBSession *session = weakSelf;
                    if (session) dispatch_async(session.queue, ^{
                        // An old interface's delayed notification must not close a reopened one.
                        if (session.interface == object) [session disconnect:"USB device disconnected"];
                    });
                }
            }];
        uint32_t speedMbps = interface ? P4NegotiatedSpeedMbps(service) : 0;
        IOObjectRelease(service);
        if (!interface) {
            if (!self.reportedOpenError) {
                uint32_t status = error ? (uint32_t)error.code : (uint32_t)kIOReturnNotFound;
                NSString *reason = [NSString stringWithFormat:@"无法打开 USB Vendor 接口 0（错误 0x%08x）。", status];
                [self emit:P4USB_ERROR token:0 data:nil reason:reason.UTF8String];
            }
            self.reportedOpenError = YES;
            continue;
        }
        NSError *outputError = nil, *inputError = nil;
        IOUSBHostPipe *output = [interface copyPipeWithAddress:0x01 error:&outputError];
        IOUSBHostPipe *input = [interface copyPipeWithAddress:0x81 error:&inputError];
        if (!output || !input) {
            if (!self.reportedOpenError) {
                NSError *pipeError = !output ? outputError : inputError;
                uint32_t status = pipeError ? (uint32_t)pipeError.code : (uint32_t)kIOReturnNotFound;
                NSString *reason = [NSString stringWithFormat:@"无法打开 USB %@ 端点（错误 0x%08x）。",
                    !output ? @"OUT 0x01" : @"IN 0x81", status];
                [self emit:P4USB_ERROR token:0 data:nil reason:reason.UTF8String];
            }
            self.reportedOpenError = YES;
            [interface destroy]; continue;
        }
        self.interface = interface; self.output = output; self.input = input;
        self.reportedOpenError = NO; self.epoch++;
        [self emit:P4USB_CONNECTED token:speedMbps data:nil reason:nullptr];
        [self readNext];
        break;
    }
    IOObjectRelease(iterator);
}
- (void)disconnect:(const char *)reason {
    if (!self.interface) return;
    self.epoch++;
    // Releasing a user-space interface does not reset the USB bus. Leave enough
    // OUT silence for the device to discard an abandoned partial logical frame.
    self.connectAfter = NSProcessInfo.processInfo.systemUptime + 3.25;
    self.input = nil; self.output = nil;
    [self.interface destroy]; self.interface = nil;
    [self.controls removeAllObjects]; self.latestVideo = nil; self.inFlight = nil; self.sending = NO;
    [self emit:P4USB_DISCONNECTED token:0 data:nil reason:reason];
}
- (void)readNext {
    if (!self.running || !self.input) return;
    uint64_t epoch = self.epoch;
    NSMutableData *buffer = [NSMutableData dataWithLength:65536];
    NSError *error = nil;
    __weak P4USBSession *weakSelf = self;
    BOOL enqueued = [self.input enqueueIORequestWithData:buffer completionTimeout:1.5 error:&error
        completionHandler:^(IOReturn status, NSUInteger actual) {
            P4USBSession *session = weakSelf;
            if (!session || !session.running || epoch != session.epoch) return;
            if (actual > buffer.length) { [session disconnect:"invalid USB read length"]; return; }
            if (status == kIOReturnSuccess || status == kIOReturnTimeout || status == kIOUSBTransactionTimeout) {
                // Bulk IN can time out after full-size USB packets without a short
                // packet/ZLP. Preserve those bytes so a split control is not lost.
                if (actual) [session emit:P4USB_BYTES token:0 data:[buffer subdataWithRange:NSMakeRange(0, actual)] reason:nullptr];
            } else {
                [session disconnect:"USB input transfer failed"]; return;
            }
            [session readNext];
        }];
    if (!enqueued) [self disconnect:"cannot submit USB input transfer"];
}
- (void)pump {
    if (!self.running || !self.output || self.sending) return;
    if (!self.inFlight) {
        if (self.controls.count) { self.inFlight = self.controls.firstObject; [self.controls removeObjectAtIndex:0]; }
        else if (self.latestVideo) { self.inFlight = self.latestVideo; self.latestVideo = nil; }
        else return;
    }
    P4Transfer *transfer = self.inFlight;
    NSUInteger count = MIN((NSUInteger)65536, transfer.data.length - transfer.offset);
    NSMutableData *chunk = [[transfer.data subdataWithRange:NSMakeRange(transfer.offset, count)] mutableCopy];
    uint64_t epoch = self.epoch;
    NSError *error = nil;
    __weak P4USBSession *weakSelf = self;
    self.sending = YES;
    // Start after the host queue wait and chunk preparation, immediately before
    // submitting the first OUT. Subsequent partial completions share this start.
    if (!transfer.outStartedNanoseconds) transfer.outStartedNanoseconds = P4MonotonicNanoseconds();
    BOOL enqueued = [self.output enqueueIORequestWithData:chunk completionTimeout:2.0 error:&error
        completionHandler:^(IOReturn status, NSUInteger actual) {
            P4USBSession *session = weakSelf;
            if (!session || !session.running || epoch != session.epoch || session.inFlight != transfer) return;
            session.sending = NO;
            // Retain chunk until completion; partial completions continue the same logical message.
            if (status != kIOReturnSuccess || actual == 0 || actual > chunk.length) {
                [session disconnect:"USB output transfer failed"]; return;
            }
            transfer.offset += actual;
            if (transfer.offset == transfer.data.length) {
                uint64_t elapsedUS = (P4MonotonicNanoseconds() - transfer.outStartedNanoseconds) / 1000;
                uint8_t metadata[8];
                for (unsigned i = 0; i < 8; i++) metadata[i] = (uint8_t)(elapsedUS >> (i * 8));
                [session emit:P4USB_SENT token:transfer.token
                    data:[NSData dataWithBytes:metadata length:sizeof(metadata)] reason:nullptr];
                session.inFlight = nil;
            }
            [session pump];
        }];
    if (!enqueued) [self disconnect:"cannot submit USB output transfer"];
}
@end

void *P4USBCreate(P4USBCallback callback, void *context, uint16_t vid, uint16_t pid) {
    P4USBSession *session = [P4USBSession new];
    session.callback = callback; session.context = context; session.vendorID = vid; session.productID = pid;
    session.queue = dispatch_queue_create("com.p4desk.usb", DISPATCH_QUEUE_SERIAL);
    session.controls = [NSMutableArray new]; session.running = YES;
    session.timer = dispatch_source_create(DISPATCH_SOURCE_TYPE_TIMER, 0, 0, session.queue);
    dispatch_source_set_timer(session.timer, DISPATCH_TIME_NOW, NSEC_PER_SEC, NSEC_PER_SEC / 5);
    __weak P4USBSession *weakSession = session;
    dispatch_source_set_event_handler(session.timer, ^{ [weakSession scan]; });
    dispatch_resume(session.timer);
    return (__bridge_retained void *)session;
}

bool P4USBEnqueue(void *handle, const uint8_t *bytes, size_t length, bool video, uint32_t token) {
    if (!handle || !bytes || length < 16 || length > 1048576 + 16) return false;
    P4USBSession *session = (__bridge P4USBSession *)handle;
    NSData *copy = [NSData dataWithBytes:bytes length:length];
    __block BOOL accepted = NO;
    dispatch_sync(session.queue, ^{
        if (!session.running || !session.interface || (!video && session.controls.count >= 64)) return;
        P4Transfer *transfer = [P4Transfer new]; transfer.data = copy; transfer.token = token; transfer.video = video;
        if (video) session.latestVideo = transfer; else [session.controls addObject:transfer];
        accepted = YES;
        // Defer pumping until after the caller receives the enqueue result.
        dispatch_async(session.queue, ^{ [session pump]; });
    });
    return accepted;
}
void P4USBClearVideo(void *handle) {
    if (!handle) return;
    P4USBSession *session = (__bridge P4USBSession *)handle;
    dispatch_sync(session.queue, ^{ session.latestVideo = nil; });
}
void P4USBReconnect(void *handle) {
    if (!handle) return;
    P4USBSession *session = (__bridge P4USBSession *)handle;
    dispatch_async(session.queue, ^{ [session disconnect:"USB session reset"]; [session scan]; });
}
void P4USBStop(void *handle) {
    if (!handle) return;
    P4USBSession *session = (__bridge_transfer P4USBSession *)handle;
    dispatch_sync(session.queue, ^{
        session.running = NO;
        dispatch_source_cancel(session.timer); session.timer = nil;
        [session disconnect:"USB stopped"];
        session.callback = nullptr; session.context = nullptr;
    });
}
