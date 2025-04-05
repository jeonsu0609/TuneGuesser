#[derive(Debug)]
pub struct Controller {
  pub count: u32,
  pub state: State,
  pub app_handler: AppHandle,
}

impl Controller {
  pub fn new(app: AppHandle) -> Play {
    Play {
      count: 0,
      state: State::Stop,
      app_handler: app,
    }
  }
}
