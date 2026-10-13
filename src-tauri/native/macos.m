#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>

@interface AwayTimeWorkspaceMonitor : NSObject
@property(nonatomic, strong) NSNotificationCenter *center;
@property(nonatomic, strong) id sleepToken;
@property(nonatomic, strong) id wakeToken;
@end
@implementation AwayTimeWorkspaceMonitor
@end

void *awaytime_watch_power(void (*callback)(int)) {
    AwayTimeWorkspaceMonitor *monitor = [AwayTimeWorkspaceMonitor new];
    monitor.center = [[NSWorkspace sharedWorkspace] notificationCenter];
    monitor.sleepToken = [monitor.center addObserverForName:NSWorkspaceWillSleepNotification object:nil
        queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *note) { (void)note; callback(0); }];
    monitor.wakeToken = [monitor.center addObserverForName:NSWorkspaceDidWakeNotification object:nil
        queue:[NSOperationQueue mainQueue] usingBlock:^(NSNotification *note) { (void)note; callback(1); }];
    return (__bridge_retained void *)monitor;
}
void awaytime_unwatch_power(void *handle) {
    AwayTimeWorkspaceMonitor *monitor = (__bridge_transfer AwayTimeWorkspaceMonitor *)handle;
    [monitor.center removeObserver:monitor.sleepToken];
    [monitor.center removeObserver:monitor.wakeToken];
}
double awaytime_idle_seconds(void) {
    return CGEventSourceSecondsSinceLastEventType(kCGEventSourceStateCombinedSessionState, kCGAnyInputEventType);
}
long awaytime_foreground_app(void) { return [[NSWorkspace sharedWorkspace] frontmostApplication].processIdentifier; }
void awaytime_restore_app(long pid) {
    NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:(pid_t)pid];
    if (app && !app.terminated) [app activateWithOptions:NSApplicationActivateIgnoringOtherApps];
}
