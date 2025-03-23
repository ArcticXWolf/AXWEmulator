use web_time::Instant;

use axwemulator_backends_chip8::{Chip8Options, Platform, create_chip8_backend};
use axwemulator_core::{backend::Backend, frontend::Frontend};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum AvailableBackends {
    #[default]
    Chip8,
    SuperChip,
}

pub struct EmulatorComponent {
    backend: Backend,
    backend_last_update: Instant,
}

impl EmulatorComponent {
    pub fn from_selection(
        backend_selection: AvailableBackends,
        frontend: &mut impl Frontend,
        rom_data: &[u8],
    ) -> Self {
        match backend_selection {
            AvailableBackends::Chip8 => Self::new_chip8(frontend, rom_data, false),
            AvailableBackends::SuperChip => Self::new_chip8(frontend, rom_data, true),
        }
    }

    fn new_chip8(frontend: &mut impl Frontend, rom_data: &[u8], super8: bool) -> Self {
        let platform = match super8 {
            false => Platform::Chip8,
            true => Platform::SuperChip,
        };

        let backend = create_chip8_backend(
            frontend,
            Chip8Options {
                platform,
                rom_data: rom_data.to_vec(),
            },
        )
        .expect("could not create backend");

        Self {
            backend,
            backend_last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        // TODO: speed boost
        let last_update_delta = self.backend_last_update.elapsed();
        self.backend_last_update = Instant::now();

        let result = self.backend.run_for(last_update_delta.into());
        if let Err(error) = result {
            panic!("{}", error);
        }
    }

    pub fn get_backend(&self) -> &Backend {
        &self.backend
    }
}
