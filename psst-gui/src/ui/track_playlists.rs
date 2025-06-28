use druid::widget::{Button, Flex, Label, List, Scroll};
use druid::{Insets, LensExt, Selector, Widget, WidgetExt, LifeCycle, LifeCycleCtx};
use std::sync::Arc;

use crate::{
    data::{AppState, Track, PlaylistLink},
    ui::theme,
    widget::{ThemeScope, MyWidgetExt},
};

pub const UPDATE_PLAYLISTS_FOR_TRACK: Selector<Arc<Track>> = Selector::new("app.track-playlists.update");

struct TrackPlaylistsController {
    track: Arc<Track>,
}

impl TrackPlaylistsController {
    fn new(track: Arc<Track>) -> Self {
        Self { track }
    }
}

impl<W: Widget<AppState>> druid::widget::Controller<AppState, W> for TrackPlaylistsController {
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppState,
        env: &druid::Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            ctx.submit_command(UPDATE_PLAYLISTS_FOR_TRACK.with(self.track.clone()));
        }
        child.lifecycle(ctx, event, data, env);
    }
}

pub fn widget(track: Arc<Track>) -> impl Widget<AppState> {
    ThemeScope::new(
        Flex::column()
            .with_child(
                Label::new(format!("Playlists containing '{}'", track.name))
                    .with_text_size(theme::TEXT_SIZE_LARGE)
                    .padding(theme::grid(2.0))
            )
            .with_flex_child(
                Scroll::new(List::new(|| {
                    Flex::row()
                        .with_flex_child(
                            Label::raw()
                                .with_text_size(theme::TEXT_SIZE_SMALL)
                                .expand_width()
                                .lens(PlaylistLink::name),
                            1.0,
                        )
                        .with_child(
                            Button::new("×")
                                .fix_size(24.0, 24.0),
                        )
                        .padding(Insets::uniform_xy(theme::grid(1.0), theme::grid(0.5)))
                }))
                .vertical()
                .lens(AppState::library.then(crate::data::Library::playlists_containing_current_track.in_arc())),
                1.0,
            )
            .background(theme::BACKGROUND_LIGHT)
    )
    .controller(TrackPlaylistsController::new(track))
    .on_command(UPDATE_PLAYLISTS_FOR_TRACK, |_, track: &Arc<Track>, data: &mut AppState| {
        data.with_library_mut(|library| {
            library.update_playlists_containing_track(&track.id);
        });
    })
} 