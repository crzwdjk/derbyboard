/*! 
This module defines request guards for accessing the current game state.
You can use these types in the parameters of the request handler, like so:
```rust,ignore
#[route("/whatever/<foo>")]
fn (foo: usize, game: Game) -> &'static str {
   let j = game.jamnum();
   ...
}
```
These request guards guarantee the existence of a current game: if there
is no current game, the request will fail.
Both of the guard types `Game` and `MutGame` implement `Deref` and the latter
implements `DerefMut` to `GameState`, so you can use them just like you would
a regular `GameState`.
 */

use std::ops::{Deref, DerefMut};
use std::sync::{RwLockReadGuard,RwLockWriteGuard};
use gamestate;
use rocket;
use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{Request,FromRequest};


/// A request guard for using the current game state, for read-only access.
pub struct Game<'a> { game: RwLockReadGuard<'a, Option<gamestate::GameState>> }

impl<'a> Deref for Game<'a> {
    type Target = gamestate::GameState;
    fn deref(&self) -> &gamestate::GameState { self.game.as_ref().unwrap() }
}

impl<'a, 'r> FromRequest<'a, 'r> for Game<'r> {
    type Error = ();
    fn from_request(_: &'a Request<'r>) -> rocket::request::Outcome<Game<'r>, ()> {
        // TODO: authentication goes here
        let game = gamestate::get_game();
        if game.is_none() {
            Outcome::Failure((Status::BadRequest, ()))
        } else {
            Outcome::Success(Game { game: game })
        }
    }
}

/// A request guard for using the current game state, for read-only access.
pub struct MutGame<'a> { game: RwLockWriteGuard<'a, Option<gamestate::GameState>> }

impl<'a> Deref for MutGame<'a> {
    type Target = gamestate::GameState;
    fn deref(&self) -> &gamestate::GameState { &self.game.as_ref().unwrap() }
}

impl<'a> DerefMut for MutGame<'a> {
    fn deref_mut(&mut self) -> &mut gamestate::GameState { self.game.as_mut().unwrap() }
}

impl<'a, 'r> FromRequest<'a, 'r> for MutGame<'r> {
    type Error = ();
    fn from_request(_: &'a Request<'r>) -> rocket::request::Outcome<MutGame<'r>, ()> {
        // TODO: authentication goes here
        let game = gamestate::get_game_mut();
        if game.is_none() {
            return rocket::Outcome::Failure((Status::BadRequest, ()))
        }
        rocket::Outcome::Success(MutGame { game: game })
    }
}

    
