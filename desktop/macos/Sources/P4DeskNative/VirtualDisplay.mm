// Private CoreGraphics declarations adapted from Stengo/DeskPad (MIT).
// Original header: Created by Khaos Tian on 2/17/21.
// Copyright (c) 2022 Bastian Andelefski. See THIRD_PARTY_NOTICES.md.
#import <Cocoa/Cocoa.h>
#import <CoreGraphics/CoreGraphics.h>
#import <objc/runtime.h>
#import "P4DeskNative.h"

@interface CGVirtualDisplayMode : NSObject
- (instancetype)initWithWidth:(unsigned int)width height:(unsigned int)height refreshRate:(CGFloat)rate;
@end
@interface CGVirtualDisplaySettings : NSObject
@property(retain, nonatomic) NSArray *modes;
@property(nonatomic) unsigned int hiDPI;
@end
@interface CGVirtualDisplayDescriptor : NSObject
@property(retain, nonatomic) NSString *name;
@property(nonatomic) unsigned int maxPixelsHigh;
@property(nonatomic) unsigned int maxPixelsWide;
@property(nonatomic) CGSize sizeInMillimeters;
@property(nonatomic) CGPoint redPrimary;
@property(nonatomic) CGPoint greenPrimary;
@property(nonatomic) CGPoint bluePrimary;
@property(nonatomic) CGPoint whitePoint;
@property(nonatomic) unsigned int serialNum;
@property(nonatomic) unsigned int serialNumber;
@property(retain, nonatomic) dispatch_queue_t queue;
@property(nonatomic) unsigned int productID;
@property(nonatomic) unsigned int vendorID;
- (void)setDispatchQueue:(dispatch_queue_t)queue;
@end
@interface CGVirtualDisplay : NSObject
@property(readonly, nonatomic) CGDirectDisplayID displayID;
- (instancetype)initWithDescriptor:(CGVirtualDisplayDescriptor *)descriptor;
- (BOOL)applySettings:(CGVirtualDisplaySettings *)settings;
@end

bool P4DisplaySymbolsAvailable(void) {
    NSArray<NSString *> *classes = @[@"CGVirtualDisplay", @"CGVirtualDisplayDescriptor",
                                     @"CGVirtualDisplayMode", @"CGVirtualDisplaySettings"];
    for (NSString *name in classes) if (!NSClassFromString(name)) return false;
    return class_getInstanceMethod(NSClassFromString(@"CGVirtualDisplay"), @selector(initWithDescriptor:)) &&
           class_getInstanceMethod(NSClassFromString(@"CGVirtualDisplay"), @selector(applySettings:)) &&
           class_getInstanceMethod(NSClassFromString(@"CGVirtualDisplayDescriptor"), @selector(setDispatchQueue:));
}

static void copy_error(char *out, size_t capacity, const char *message) {
    if (out && capacity) snprintf(out, capacity, "%s", message);
}

void *P4DisplayCreate(uint32_t width, uint32_t height, char *error, size_t capacity) {
    if (![NSThread isMainThread]) { copy_error(error, capacity, "display creation requires main thread"); return nullptr; }
    if (width != 1024 || height != 600 || !P4DisplaySymbolsAvailable()) {
        copy_error(error, capacity, "virtual display API unavailable or unsupported dimensions"); return nullptr;
    }
    @try {
        // Instantiate via NSClassFromString so a removed private class does not prevent launch.
        CGVirtualDisplayDescriptor *descriptor = [[NSClassFromString(@"CGVirtualDisplayDescriptor") alloc] init];
        descriptor.name = @"P4 Desk";
        descriptor.maxPixelsWide = width;
        descriptor.maxPixelsHigh = height;
        // Report a conventional desktop density so macOS keeps the requested 1:1 logical mode.
        descriptor.sizeInMillimeters = CGSizeMake(width * 25.4 / 96.0, height * 25.4 / 96.0);
        descriptor.redPrimary = CGPointMake(0.64, 0.33);
        descriptor.greenPrimary = CGPointMake(0.30, 0.60);
        descriptor.bluePrimary = CGPointMake(0.15, 0.06);
        descriptor.whitePoint = CGPointMake(0.3127, 0.3290);
        descriptor.vendorID = 0x5044;
        descriptor.productID = 0x0007;
        // Stable identity keeps the user's display arrangement and reuses one ICC profile.
        const unsigned int stableIdentity = 0x50344431;
        if ([descriptor respondsToSelector:@selector(setSerialNumber:)]) descriptor.serialNumber = stableIdentity;
        else descriptor.serialNum = stableIdentity;
        dispatch_queue_t displayQueue = dispatch_queue_create("com.p4desk.virtual-display", DISPATCH_QUEUE_SERIAL);
        if ([descriptor respondsToSelector:@selector(setQueue:)]) descriptor.queue = displayQueue;
        else [descriptor setDispatchQueue:displayQueue];
        CGVirtualDisplay *display = [[NSClassFromString(@"CGVirtualDisplay") alloc] initWithDescriptor:descriptor];
        if (!display || display.displayID == 0) { copy_error(error, capacity, "virtual display creation rejected"); return nullptr; }
        CGVirtualDisplaySettings *settings = [[NSClassFromString(@"CGVirtualDisplaySettings") alloc] init];
        settings.hiDPI = 0;
        settings.modes = @[[[NSClassFromString(@"CGVirtualDisplayMode") alloc] initWithWidth:width height:height refreshRate:60.0]];
        if (![display applySettings:settings]) { copy_error(error, capacity, "virtual display mode rejected"); return nullptr; }
        return (__bridge_retained void *)display;
    } @catch (NSException *exception) {
        copy_error(error, capacity, "virtual display API rejected configuration");
        return nullptr;
    }
}

uint32_t P4DisplayID(void *handle) { return handle ? ((__bridge CGVirtualDisplay *)handle).displayID : 0; }
void P4DisplayDestroy(void *handle) {
    if (!handle) return;
    // ARC release removes the display from WindowServer.
    CGVirtualDisplay *display = (__bridge_transfer CGVirtualDisplay *)handle;
    display = nil;
}
