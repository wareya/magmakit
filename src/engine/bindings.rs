use std::rc::Rc;
use std::cell::RefCell;

use super::*;
use super::render::*;

impl Engine {
    pub fn mouse_cursor_disable(&mut self)
    {
        self.renderer.display.gl_window().window().hide_cursor(true);
    }
    pub fn mouse_cursor_enable(&mut self)
    {
        self.renderer.display.gl_window().window().hide_cursor(false);
    }
    
    pub fn set_framerate(&mut self, mut framerate : f64)
    {
        if framerate < 0.001
        {
            framerate = 0.001;
        }
        self.target_frametime = 1.0/framerate;
        self.framelimiter_reference = self.framelimiter_reset_reference_time;
        self.framelimiter_count = 0;
    }
    pub fn set_frametime(&mut self, mut frametime : f64)
    {
        if frametime < 0.001
        {
            frametime = 0.001
        }
        self.target_frametime = frametime;
        self.framelimiter_reference = self.framelimiter_reset_reference_time;
        self.framelimiter_count = 0;
    }
    pub fn get_target_framerate(&mut self) -> f64
    {
        1.0/self.target_frametime
    }
    pub fn get_target_frametime(&mut self) -> f64
    {
        self.target_frametime
    }
    pub fn get_immediate_framerate(&mut self) -> f64
    {
        1.0/self.framelimiter_delta
    }
    pub fn get_smooth_framerate(&mut self) -> f64
    {
        1.0/(self.recent_deltas.iter().sum::<f64>() / self.recent_deltas.len() as f64)
    }
    pub fn get_perceptual_framerate(&mut self) -> f64
    {
        let total_delta = self.recent_deltas.iter().sum::<f64>() as f64;
        let broken_deltas = self.recent_deltas.iter().map(|x| x*x/total_delta/total_delta).collect::<Vec<_>>();
        let avg_broken_delta = total_delta * broken_deltas.iter().sum::<f64>();
        1.0/avg_broken_delta
    }
    pub fn get_frame_delta_secs(&mut self) -> f64
    {
        self.framelimiter_delta
    }
    pub fn get_frame_delta_msecs(&mut self) -> f64
    {
        self.framelimiter_delta*1000.0
    }
    pub fn get_frame_delta_frames(&mut self, delta : f64) -> f64
    {
        delta/self.framelimiter_delta
    }
}