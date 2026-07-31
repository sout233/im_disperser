#import <AppKit/AppKit.h>
#import <AudioToolbox/AudioToolbox.h>
#import <AudioUnit/AUCocoaUIView.h>
#import <CoreFoundation/CoreFoundation.h>

#include <stdio.h>

static OSType fourcc(const char *value) {
    return ((OSType)(unsigned char)value[0] << 24) |
           ((OSType)(unsigned char)value[1] << 16) |
           ((OSType)(unsigned char)value[2] << 8) |
           (OSType)(unsigned char)value[3];
}

static int fail(const char *message, OSStatus status) {
    fprintf(stderr, "%s: %d\n", message, (int)status);
    return 1;
}

int main(void) {
    @autoreleasepool {
        AudioComponentDescription description = {
            .componentType = fourcc("aufx"),
            .componentSubType = fourcc("IMDS"),
            .componentManufacturer = fourcc("SOUT"),
            .componentFlags = 0,
            .componentFlagsMask = 0,
        };
        AudioComponent component = AudioComponentFindNext(NULL, &description);
        AudioComponentInstance instance = NULL;
        AudioUnitCocoaViewInfo view_info = {};
        UInt32 view_info_size = sizeof(view_info);
        Boolean writable = false;
        OSStatus status;

        if (component == NULL) {
            fprintf(stderr, "AU component was not found.\n");
            return 1;
        }

        status = AudioComponentInstanceNew(component, &instance);
        if (status != noErr || instance == NULL) {
            return fail("AudioComponentInstanceNew failed", status);
        }

        status = AudioUnitGetPropertyInfo(instance, kAudioUnitProperty_CocoaUI,
                                          kAudioUnitScope_Global, 0,
                                          &view_info_size, &writable);
        if (status != noErr || view_info_size != sizeof(view_info)) {
            AudioComponentInstanceDispose(instance);
            return fail("Cocoa UI property is unavailable", status);
        }

        status = AudioUnitGetProperty(instance, kAudioUnitProperty_CocoaUI,
                                      kAudioUnitScope_Global, 0,
                                      &view_info, &view_info_size);
        if (status != noErr || view_info.mCocoaAUViewBundleLocation == NULL ||
            view_info.mCocoaAUViewClass[0] == NULL) {
            AudioComponentInstanceDispose(instance);
            return fail("Cocoa UI property could not be read", status);
        }

        NSURL *bundle_url = (__bridge NSURL *)view_info.mCocoaAUViewBundleLocation;
        NSString *class_name = (__bridge NSString *)view_info.mCocoaAUViewClass[0];
        NSBundle *bundle = [NSBundle bundleWithURL:bundle_url];
        BOOL loaded = [bundle load];
        Class cocoa_ui_class = NSClassFromString(class_name);
        if (!loaded || cocoa_ui_class == Nil ||
            ![cocoa_ui_class conformsToProtocol:@protocol(AUCocoaUIBase)]) {
            CFRelease(view_info.mCocoaAUViewBundleLocation);
            AudioComponentInstanceDispose(instance);
            fprintf(stderr, "Cocoa UI class could not be loaded: %s\n",
                    class_name.UTF8String);
            return 1;
        }

        id<AUCocoaUIBase> provider = [[cocoa_ui_class alloc] init];
        NSView *view = [provider uiViewForAudioUnit:instance
                                           withSize:NSMakeSize(800.0, 600.0)];
        if (view == nil) {
            CFRelease(view_info.mCocoaAUViewBundleLocation);
            AudioComponentInstanceDispose(instance);
            fprintf(stderr, "Cocoa UI provider returned a nil NSView.\n");
            return 1;
        }

        [NSApplication sharedApplication];
        NSWindow *host_window = [[NSWindow alloc]
            initWithContentRect:NSMakeRect(0.0, 0.0, 800.0, 600.0)
                      styleMask:NSWindowStyleMaskBorderless
                        backing:NSBackingStoreBuffered
                          defer:NO];
        NSView *host_view = [[NSView alloc] initWithFrame:NSMakeRect(0.0, 0.0, 800.0, 600.0)];
        host_window.contentView = host_view;
        [host_view addSubview:view];
        [host_view layoutSubtreeIfNeeded];

        if (view.window != host_window) {
            [view removeFromSuperview];
            [host_window close];
            CFRelease(view_info.mCocoaAUViewBundleLocation);
            AudioComponentInstanceDispose(instance);
            fprintf(stderr, "Cocoa UI view was not attached to the host window.\n");
            return 1;
        }

        printf("Cocoa UI passed: class=%s frame=%.0fx%.0f attached=yes\n",
               class_name.UTF8String, view.frame.size.width, view.frame.size.height);
        [view removeFromSuperview];
        [host_window close];
        CFRelease(view_info.mCocoaAUViewBundleLocation);
        AudioComponentInstanceDispose(instance);
        return 0;
    }
}
