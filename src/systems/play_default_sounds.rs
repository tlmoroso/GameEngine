use specs::{System, Write, Join, WriteStorage};
use crate::globals::AudioController;
use crate::components::audibles::default_sound::DefaultSound;
use kira::instance::{InstanceSettings, StopInstanceSettings};

pub struct PlayDefaultSounds;

impl<'a> System<'a> for PlayDefaultSounds {
    type SystemData = (
        Write<'a, AudioController>,
        WriteStorage<'a, DefaultSound>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut audio_controller, mut default_sounds) = data;

        for default_sound in (&mut default_sounds).join() {
            if default_sound.play_flag && default_sound.instance_id.is_none(){
                let sound_id = audio_controller.audio_lib.0.get(default_sound.sound_name.as_str())
                    .expect(format!("ERROR: Failed to find SoundID for name: {:#?} in AudioDict: {:#?}", default_sound.sound_name, audio_controller.audio_lib).as_str());

                let mut audio_manager = audio_controller.audio_manager.write()
                    .expect("ERROR: Failed to acquire write lock for audio manager");

                let instance_id = audio_manager.play(sound_id.clone(), InstanceSettings::new());
                if instance_id.is_err() {
                    println!("Error playing sound_id: {:#?} from default_sound: {:#?}", sound_id, default_sound.sound_name);
                }

                default_sound.instance_id = instance_id.ok();
            } else if !default_sound.play_flag && default_sound.instance_id.is_some() {
                let mut audio_manager = audio_controller.audio_manager.write()
                    .expect("ERROR: Failed to acquire write lock for audio manager");

                audio_manager.stop_instance(default_sound.instance_id.unwrap(), StopInstanceSettings::new())
                    .expect(format!("Failed to stop instance: {:#?} from default_sound: {:#?}", default_sound.instance_id, default_sound).as_str());
            }
        }
    }
}