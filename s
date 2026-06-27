[1mdiff --git a/os/gui-app-onboarding/src/main.rs b/os/gui-app-onboarding/src/main.rs[m
[1mindex 812c6a7..8ca0993 100644[m
[1m--- a/os/gui-app-onboarding/src/main.rs[m
[1m+++ b/os/gui-app-onboarding/src/main.rs[m
[36m@@ -150,6 +150,17 @@[m [mfn on_startup(state: StoredValue<AppState>) {[m
         log::info!("no update state detected");[m
         match master_key_state {[m
             MasterKeyState::Onboarding => {}[m
[32m+[m[32m            // SIM FIX: a fully-provisioned device (seed + PIN) must NOT be wiped on[m
[32m+[m[32m            // boot. Real hardware persists onboarding-complete and skips this app[m
[32m+[m[32m            // entirely; the hosted sim doesn't persist that flag, so on_startup[m
[32m+[m[32m            // re-runs and the old catch-all factory-reset a Normal wallet. Treat[m
[32m+[m[32m            // Normal as "already set up": finish and go to the launcher.[m
[32m+[m[32m            MasterKeyState::Normal => {[m
[32m+[m[32m                log::info!("device already provisioned (Normal) — skipping onboarding, launching");[m
[32m+[m[32m                state.borrow_mut().finished = true;[m
[32m+[m[32m                state.borrow().gui.switch_to_launcher().ok();[m
[32m+[m[32m                return;[m
[32m+[m[32m            }[m
             _ => {[m
                 // if we have reached this branch, then we have rebooted mid-way through onboarding[m
                 // which is un-recoverable. we must factory reset restart onboarding[m
[1mdiff --git a/os/gui-app-onboarding/ui/pages/connect-wallet/page.slint b/os/gui-app-onboarding/ui/pages/connect-wallet/page.slint[m
[1mindex e43d693..ac2f6e6 100644[m
[1m--- a/os/gui-app-onboarding/ui/pages/connect-wallet/page.slint[m
[1m+++ b/os/gui-app-onboarding/ui/pages/connect-wallet/page.slint[m
[36m@@ -53,7 +53,7 @@[m [mexport component ConnectWalletPage inherits OnboardingBasePage {[m
         ButtonSection {[m
             Button {[m
                 label: TR2.lookup(TrId.CommonButtonFinish);[m
[31m-                visible: wallet-connected;[m
[32m+[m[32m                visible: true;  /* SIM: always allow finishing without phone */[m
 [m
                 clicked => {[m
                     OnboardingCallbacks.finish-onboarding();[m
[1mdiff --git a/os/gui-app-onboarding/ui/pages/welcome/page.slint b/os/gui-app-onboarding/ui/pages/welcome/page.slint[m
[1mindex 9326f72..8d46df4 100644[m
[1m--- a/os/gui-app-onboarding/ui/pages/welcome/page.slint[m
[1m+++ b/os/gui-app-onboarding/ui/pages/welcome/page.slint[m
[36m@@ -60,7 +60,7 @@[m [mexport component WelcomePage inherits OnboardingShieldPage {[m
                 label: TR2.lookup(TrId.CommonButtonGetStarted);[m
                 importance: primary;[m
                 clicked => {[m
[31m-                    Navigate.scan-qr({ });[m
[32m+[m[32m                    Navigate.set-pin-info({ });  /* SIM: skip phone scan, go to PIN+create */[m
                 }[m
             }[m
         }[m
