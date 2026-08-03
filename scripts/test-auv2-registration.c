#include <AudioToolbox/AudioToolbox.h>
#include <CoreFoundation/CoreFoundation.h>
#include <stdio.h>
#include <string.h>

static OSType fourcc(const char *value) {
    return ((OSType)(unsigned char)value[0] << 24) |
           ((OSType)(unsigned char)value[1] << 16) |
           ((OSType)(unsigned char)value[2] << 8) |
           (OSType)(unsigned char)value[3];
}

static int parse_fourcc(const char *label, const char *value, OSType *result) {
    if (strlen(value) != 4) {
        fprintf(stderr, "%s must be exactly four characters: <%s>\n", label, value);
        return 0;
    }

    *result = fourcc(value);
    return 1;
}

int main(int argc, char *argv[]) {
    const char *type_string = "aufx";
    const char *subtype_string = "IMDS";
    const char *manufacturer_string = "SOUT";
    OSType type;
    OSType subtype;
    OSType manufacturer;
    AudioComponentDescription description = {0};
    AudioComponent component;
    AudioComponentInstance instance = NULL;
    CFStringRef component_name = NULL;
    UInt32 version = 0;
    OSStatus status;
    char name_buffer[512] = {0};

    if (argc == 4) {
        type_string = argv[1];
        subtype_string = argv[2];
        manufacturer_string = argv[3];
    } else if (argc != 1) {
        fprintf(stderr, "Usage: %s [type subtype manufacturer]\n", argv[0]);
        return 64;
    }

    if (!parse_fourcc("type", type_string, &type) ||
        !parse_fourcc("subtype", subtype_string, &subtype) ||
        !parse_fourcc("manufacturer", manufacturer_string, &manufacturer)) {
        return 64;
    }

    description.componentType = type;
    description.componentSubType = subtype;
    description.componentManufacturer = manufacturer;
    component = AudioComponentFindNext(NULL, &description);
    if (component == NULL) {
        fprintf(stderr, "AudioComponentFindNext did not find '%s'/'%s'/'%s'.\n",
                type_string, subtype_string, manufacturer_string);
        return 1;
    }

    status = AudioComponentCopyName(component, &component_name);
    if (status != noErr || component_name == NULL ||
        !CFStringGetCString(component_name, name_buffer, sizeof(name_buffer),
                            kCFStringEncodingUTF8)) {
        fprintf(stderr, "AudioComponentCopyName failed: %d\n", (int)status);
        if (component_name != NULL) {
            CFRelease(component_name);
        }
        return 1;
    }
    CFRelease(component_name);

    status = AudioComponentGetVersion(component, &version);
    if (status != noErr) {
        fprintf(stderr, "AudioComponentGetVersion failed: %d\n", (int)status);
        return 1;
    }

    status = AudioComponentInstanceNew(component, &instance);
    if (status != noErr || instance == NULL) {
        fprintf(stderr, "AudioComponentInstanceNew failed: %d\n", (int)status);
        return 1;
    }

    status = AudioUnitInitialize(instance);
    if (status != noErr) {
        fprintf(stderr, "AudioUnitInitialize failed: %d\n", (int)status);
        AudioComponentInstanceDispose(instance);
        return 1;
    }

    AudioUnitUninitialize(instance);
    AudioComponentInstanceDispose(instance);
    printf("Registered AU: %s (version %u), instance creation and initialization passed.\n",
           name_buffer, (unsigned int)version);
    return 0;
}
