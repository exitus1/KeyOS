// SPDX-FileCopyrightText: 2024 Foundation Devices, Inc. <hello@foundation.xyz>
// SPDX-License-Identifier: GPL-3.0-or-later

use gui_server_api::{
    error::NavigationError,
    msg::{FinishResponse, GetPendingNavRequest, LoginSuccess, NavigateTo, NavigationCancel, ShowModal},
};
use server::{Archive, ArchiveAsyncHandler, ArchiveHandler, ArchiveRequest, ScalarHandler, ServerContext};
use xous::PID;

use crate::Gui;

impl ArchiveAsyncHandler<ShowModal> for Gui {
    fn default_response() -> <ShowModal as Archive>::Response { Err(NavigationError::InternalError) }

    fn handle(&mut self, request: ArchiveRequest<ShowModal>, _context: &mut ServerContext<Self>) {
        self.handle_show_modal_request(request);
    }
}

impl ArchiveAsyncHandler<NavigateTo> for Gui {
    fn default_response() -> <NavigateTo as Archive>::Response { Err(NavigationError::InternalError) }

    fn handle(&mut self, request: ArchiveRequest<NavigateTo>, _context: &mut ServerContext<Self>) {
        self.handle_navigate_to_request(request);
    }
}

impl ArchiveHandler<FinishResponse> for Gui {
    fn handle(
        &mut self,
        msg: FinishResponse,
        sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> <FinishResponse as Archive>::Response {
        if Some(sender) == self.active_app_pid() {
            self.respond_to_nav_request(Ok(msg))
        } else {
            log::warn!("FinishResponse got from invalid pid: {sender}");
        }
    }
}

impl ScalarHandler<NavigationCancel> for Gui {
    fn handle(&mut self, _msg: NavigationCancel, sender: PID, _context: &mut ServerContext<Self>) {
        if Some(sender) == self.active_app_pid() {
            self.respond_to_nav_request(Err(NavigationError::CanceledByUser));
        } else {
            log::warn!("NavigationCancel got from invalid pid: {sender}");
        }
    }
}

impl ArchiveHandler<GetPendingNavRequest> for Gui {
    fn handle(
        &mut self,
        _msg: GetPendingNavRequest,
        sender: PID,
        _context: &mut ServerContext<Self>,
    ) -> <GetPendingNavRequest as Archive>::Response {
        if Some(sender) == self.active_app_pid() {
            self.get_pending_nav_request()
        } else {
            log::warn!("GetPendingNavRequest got from invalid pid: {sender}");
            None
        }
    }
}

impl ScalarHandler<LoginSuccess> for Gui {
    fn handle(&mut self, _msg: LoginSuccess, sender: PID, _context: &mut ServerContext<Self>) {
        if !self.app_registry.is_lock_screen_app(sender) {
            return;
        }

        self.unlock();
    }
}
