use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_ecs::prelude::{Res, ResMut, Resource, Trigger};
use bevy_minibuffer::prelude::*;
use std::borrow::Cow;
use std::marker::PhantomData;
use trie_rs::map::Trie;

#[derive(Resource)]
pub(crate) struct Inspectors<M: Send + Sync + 'static> {
    pub(crate) names: Trie<u8, usize>,
    pub(crate) visible: Vec<bool>,
    marker: PhantomData<M>,
}

pub(crate) struct InspectorPlugins<M> {
    plugins: Option<PluginGroupBuilder>,
    names: Vec<String>,
    marker: PhantomData<M>,
}

impl<M: Send + Sync + 'static> InspectorPlugins<M> {
    pub(crate) fn add_inspector<F: Fn(usize, &mut Self)>(
        &mut self,
        name: String,
        add_plugin_fn: F,
    ) {
        let index = self.names.len();
        self.names.push(name);
        add_plugin_fn(index, self)
    }

    pub(crate) fn add_plugin<T: Plugin>(&mut self, plugin: T) {
        let builder = self.plugins.take().expect("plugin builder");
        self.plugins = Some(builder.add(plugin));
    }

    /// Return true if the inspector is marked as visible.
    pub(crate) fn visible(index: usize) -> impl Fn(Res<Inspectors<M>>) -> bool {
        move |inspectors: Res<Inspectors<M>>| {
            inspectors
                .into_inner()
                .visible
                .get(index)
                .copied()
                .unwrap_or(false)
        }
    }

    // We could subsistute the inspector with this generic, but it felt a bit
    // too rigid to go for this early.
    #[allow(dead_code)]
    pub(crate) fn inspector(
        prompt: impl Into<Cow<'static, str>>,
        none_msg: impl Into<Cow<'static, str>>,
    ) -> impl Fn(Res<Inspectors<M>>, Minibuffer) {
        let prompt = prompt.into();
        let none_msg = none_msg.into();
        move |inspectors: Res<Inspectors<M>>, mut minibuffer: Minibuffer| {
            if !inspectors.visible.is_empty() {
                minibuffer
                    .prompt_map(prompt.clone(), inspectors.names.clone())
                    .observe(
                        |mut trigger: Trigger<Completed<usize>>,
                         mut inspectors: ResMut<Inspectors<M>>| {
                            if let Ok(index) = trigger.event_mut().take_result().unwrap() {
                                inspectors.visible[index] = !inspectors.visible[index];
                            }
                        },
                    );
            } else {
                minibuffer.message(none_msg.clone());
            }
        }
    }
}

impl<M: Send + Sync + 'static> Default for InspectorPlugins<M> {
    fn default() -> Self {
        Self {
            plugins: Some(PluginGroupBuilder::start::<Self>()),
            names: vec![],
            marker: PhantomData,
        }
    }
}

impl<M: Send + Sync + 'static> PluginGroup for InspectorPlugins<M> {
    fn build(mut self) -> PluginGroupBuilder {
        let builder = self.plugins.take().expect("plugin builder");
        // self.warn_on_unused_acts();
        builder.add(move |app: &mut App| {
            let count = self.names.len();
            let trie = Trie::from_iter(
                self.names
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, name)| (name, index)),
            );
            app.insert_resource(Inspectors {
                names: trie,
                visible: vec![false; count],
                marker: PhantomData::<M>,
            });
        })
    }
}

// fn state_inspector_plugin<A: FreelyMutableState + Reflect>(index: usize) -> impl Fn(&mut App) {
//     move |app: &mut App| {
//         app.add_plugins(StateInspectorPlugin::<A>::default()
//                         .run_if(in_state(PromptState::Visible).and(state_visible(index)))
//         );
//     }
// }
