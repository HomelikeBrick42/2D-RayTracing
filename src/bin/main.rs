use anyhow::Result;
use game::Game;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Game")
        .build(&event_loop)
        .unwrap();

    let mut game = pollster::block_on(Game::new(window))?;
    event_loop.run(move |event, _, control_flow| {
        if let Err(err) = game.handle_event(event, control_flow) {
            eprintln!("{err}");
            *control_flow = ControlFlow::ExitWithCode(1);
        }
    });
}
