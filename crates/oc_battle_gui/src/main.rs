// use bevy::prelude::*;

// pub const WORLD_WIDTH: usize = 1024;
// pub const WORLD_HEIGHT: usize = 1024;
// pub const SOLDIERS_COUNT: usize = 2048;

// mod performances {
//     use std::ops::Deref;
//     use std::sync::atomic::AtomicU64;
//     use std::sync::atomic::Ordering;

//     use bevy::prelude::*;

//     pub struct PerformanceTrackingPlugin;

//     impl Plugin for PerformanceTrackingPlugin {
//         fn build(&self, app: &mut App) {
//             app.insert_resource(PerformancesResource(Performances {
//                 tick: AtomicU64::new(0),
//             }));
//             app.insert_resource(PerfTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
//             app.add_systems(Update, print_perf);
//         }
//     }

//     fn print_perf(time: Res<Time>, mut timer: ResMut<PerfTimer>, perf: Res<PerformancesResource>) {
//         if timer.0.tick(time.delta()).just_finished() {
//             let count = perf.ticks();
//             println!("{count} tick/s");
//             perf.reset();
//         }
//     }

//     #[derive(Resource)]
//     struct PerfTimer(Timer);

//     #[derive(Debug)]
//     pub struct Performances {
//         pub tick: AtomicU64,
//     }

//     #[derive(Debug, Resource)]
//     pub struct PerformancesResource(pub Performances);

//     impl Deref for PerformancesResource {
//         type Target = Performances;

//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     impl Performances {
//         pub fn ticks(&self) -> u64 {
//             self.tick.load(std::sync::atomic::Ordering::Relaxed)
//         }

//         pub fn incr(&self) {
//             self.tick.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//         }

//         pub fn reset(&self) {
//             self.tick.swap(0, Ordering::Relaxed);
//         }
//     }
// }

// mod world {
//     use bevy::prelude::*;

//     use crate::{
//         WORLD_HEIGHT, WORLD_WIDTH,
//         utils::d2::{Xy, XyIndex},
//     };

//     pub struct WorldPlugin;

//     impl Plugin for WorldPlugin {
//         fn build(&self, app: &mut App) {
//             let world = World(vec![tile::Tile::ShortGrass; WORLD_WIDTH * WORLD_HEIGHT]);
//             app.insert_resource(WorldResource(world));
//         }
//     }

//     #[derive(Resource)]
//     pub struct WorldResource(World);

//     impl std::ops::Deref for WorldResource {
//         type Target = World;

//         fn deref(&self) -> &Self::Target {
//             &self.0
//         }
//     }

//     pub mod tile {
//         #[derive(Debug, Clone)]
//         pub enum Tile {
//             ShortGrass,
//         }
//     }
// }

// mod physics {
//     use bevy::prelude::*;

//     use crate::utils::d2::Xy;

//     #[derive(Debug, Component)]
//     pub struct Position(pub Xy);
// }

// mod entities {
//     use bevy::prelude::*;

//     use crate::{
//         SOLDIERS_COUNT, WORLD_HEIGHT,
//         entities::soldier::{Behavior, Soldier, SoldierIndex, UpdateSoldier},
//         performances::PerformancesResource,
//         physics::Position,
//         utils::d2::{Xy, XyIndex},
//         world::{self, WorldResource},
//     };

//     pub struct EntitiesPlugin;

//     impl Plugin for EntitiesPlugin {
//         fn build(&self, app: &mut App) {
//             app.insert_resource(Soldiers(Vec::with_capacity(SOLDIERS_COUNT)));
//             app.add_systems(Startup, init_soldiers);
//             app.add_systems(Update, move_soldiers);
//             app.add_observer(on_update_soldier);
//         }
//     }

//     #[derive(Resource)]
//     pub struct Soldiers(pub Vec<Entity>);

//     fn init_soldiers(mut commands: Commands, mut soldiers: ResMut<Soldiers>) {
//         for i in 0..SOLDIERS_COUNT {
//             let entity = commands.spawn((
//                 Soldier,
//                 SoldierIndex(i),
//                 Position(Xy::from(XyIndex(i))),
//                 Behavior::MovingSouth,
//             ));
//             soldiers.0.push(entity.id());
//         }
//     }

//     fn move_soldiers(
//         mut commands: Commands,
//         positions: Query<(&SoldierIndex, &Position, &Behavior), With<Soldier>>,
//         world: Res<WorldResource>,
//         perf: Res<PerformancesResource>,
//     ) {
//         for (i, position, behavior) in positions {
//             perf.incr();

//             // Simulate usage of read-only world resource
//             let Some(world::tile::Tile::ShortGrass) = world.tile(position.0) else {
//                 continue;
//             };

//             let (x, y): (usize, usize) = position.0.into();
//             let (next_position, next_behavior) = match behavior {
//                 Behavior::MovingNorth => {
//                     if x == 0 {
//                         (position.0, Behavior::MovingSouth)
//                     } else {
//                         (Xy(x - 1, y), Behavior::MovingNorth)
//                     }
//                 }
//                 Behavior::MovingSouth => {
//                     if x == WORLD_HEIGHT - 1 {
//                         (position.0, Behavior::MovingNorth)
//                     } else {
//                         (Xy(x + 1, y), Behavior::MovingSouth)
//                     }
//                 }
//             };

//             commands.trigger(UpdateSoldier(*i, next_position, next_behavior));
//         }
//     }

//     fn on_update_soldier(
//         update: On<UpdateSoldier>,
//         soldiers: Res<Soldiers>,
//         mut positions: Query<&mut Position>,
//         mut behaviors: Query<&mut Behavior>,
//     ) {
//         let (i, new_position, new_behavior) = (update.0, update.1, update.2);
//         let mut soldier = soldiers.0[i.0];
//     }

//     mod soldier {
//         use bevy::prelude::*;

//         use crate::utils::d2::Xy;

//         #[derive(Debug, Component)]
//         pub struct Soldier;

//         #[derive(Debug, Clone, Copy, Component)]
//         pub struct SoldierIndex(pub usize);

//         #[derive(Debug, Clone, Copy, Component)]
//         pub enum Behavior {
//             MovingNorth,
//             MovingSouth,
//         }

//         #[derive(Event)]
//         pub struct UpdateSoldier(pub SoldierIndex, pub Xy, pub Behavior);
//     }
// }
// fn main() {
//     App::new()
//         .add_plugins(DefaultPlugins)
//         .add_plugins(performances::PerformanceTrackingPlugin)
//         .add_plugins(world::WorldPlugin)
//         .add_plugins(entities::EntitiesPlugin)
//         .run();
// }

use std::net::SocketAddr;
use std::time::Duration;

use clap::Parser;
use message_io::network::{NetEvent, Transport};
use message_io::node::{self, NodeEvent};
use oc_network::{ArchivedToClient, ToClient, ToServer};
use rkyv::api::low::deserialize;
use rkyv::rancor::Error;
use rkyv::util::AlignedVec;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod network;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(default_value = "0.0.0.0:6589")]
    pub server: SocketAddr,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();

    let host = args.server;
    let (handler, listener) = node::split::<()>();
    let (server, _) = handler
        .network()
        .connect(Transport::FramedTcp, host)
        .unwrap();

    std::thread::spawn(move || {
        listener.for_each(move |event| match event {
            NodeEvent::Network(event) => match event {
                NetEvent::Connected(_endpoint, _ok) => {
                    tracing::info!("Connected to server ({host})");
                }
                NetEvent::Accepted(_, _) => unreachable!(),
                NetEvent::Message(_, bytes_) => {
                    let mut bytes: AlignedVec = rkyv::util::AlignedVec::with_capacity(bytes_.len());
                    bytes.extend_from_slice(bytes_);
                    let message = rkyv::access::<ArchivedToClient, Error>(&bytes).unwrap(); // TODO
                    let message = deserialize::<ToClient, Error>(message).unwrap(); // TODO
                    tracing::trace!(name="network-received", message = ?message);
                }
                NetEvent::Disconnected(_endpoint) => (),
            },
            NodeEvent::Signal(_signal) => {}
        });
    });

    loop {
        std::thread::sleep(Duration::from_secs(1));
        let bytes = rkyv::to_bytes::<Error>(&ToServer::Hello).unwrap(); // TODO
        handler.network().send(server, &bytes);
    }

    Ok(())
}
