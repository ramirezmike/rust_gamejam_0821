use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};


pub struct AudioState {
    pub channels: HashMap<AudioChannel, ChannelAudioState>,
    pub sound_channel: AudioChannel,
    pub music_channel: AudioChannel,
    pub music_handle: Handle<AudioSource>,
}


pub struct ChannelAudioState {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f32,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            volume: 0.6,
            stopped: true,
            loop_started: false,
            paused: false,
        }
    }
}

impl AudioState {
    pub fn new(asset_server: &Res<AssetServer>) -> AudioState {
        let mut channels = HashMap::new();
        let sound_channel = AudioChannel::new("first".to_owned());
        let music_channel = AudioChannel::new("music".to_owned());

        channels.insert(
            sound_channel.clone(),
            ChannelAudioState::default(),
        );
        channels.insert(
            music_channel.clone(),
            ChannelAudioState::default(),
        );

        AudioState {
            sound_channel,
            music_channel,
            channels,
            music_handle: asset_server.load("music/music.wav"),
        }
    }

    pub fn start_music_channels(&mut self, audio: &Res<Audio>) {
        AudioState::start_music_channel(&mut self.channels, audio, &self.music_handle, &self.music_channel);
    }

    fn start_music_channel(
        channels: &mut HashMap<AudioChannel, ChannelAudioState>,
        audio: &Res<Audio>, 
        handle: &Handle<AudioSource>, 
        channel: &AudioChannel
    ) {
        let mut channel_audio_state = channels.get_mut(channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        audio.set_volume_in_channel(0.0, channel);
        audio.play_looped_in_channel(handle.clone(), channel);
    }
}

pub fn play_music(
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
//    audio.set_volume_in_channel(1.0, &audio_state.music_channel);
}
