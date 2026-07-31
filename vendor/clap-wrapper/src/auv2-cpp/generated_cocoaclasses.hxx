#pragma once

#define CLAP_WRAPPER_TIMER_CALLBACK timerCallback_wrapAsAUV2_cocoaUI
#define CLAP_WRAPPER_FILL_AUCV fillAUCV_wrapAsAUV2_cocoaUI
#define CLAP_WRAPPER_EDITOR_NAME "Plugin View"
#include "detail/auv2/wrappedview.asinclude.mm"

bool fillAudioUnitCocoaView(AudioUnitCocoaViewInfo *viewInfo, std::shared_ptr<Clap::Plugin> _plugin)
{
    return fillAUCV_wrapAsAUV2_cocoaUI(viewInfo);
}