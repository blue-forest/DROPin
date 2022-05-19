/*     _              _ _
 *  __| |_ _ ___ _ __( |_)_ _
 * / _` | '_/ _ \ '_ \/| | ' \
 * \__,_|_| \___/ .__/ |_|_||_| dropin-compiler - WebAssembly
 *              |_|
 * Copyright © 2019-2022 Blue Forest
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

use std::fmt::{Display, Error, Formatter};
use std::fs::create_dir;

use crate::interactive::{Cli, Command};
use crate::utils::validate_name;

pub struct Add;

impl Display for Add {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		"add".fmt(f)
	}
}

impl Command for Add {
	fn run(&self, cli: &mut Cli) -> u32 {
		let (owner_name, owner_path) = loop {
			let owner_name: String = Input::with_theme(&ColorfulTheme::default())
				.with_prompt("Owner name for your recipes ? (leave empty to cancel)")
				.allow_empty(true)
				.interact_text()
				.unwrap();
			if owner_name.is_empty() {
				return 0;
			}
			let owner_path = cli.root.join(&owner_name);
			if let Err(err) = validate_name(&owner_path, &owner_name) {
				println!("{}", err);
				continue;
			}
			break (owner_name, owner_path);
		};
		create_dir(&owner_path).unwrap();
		println!("Owner {} created", owner_name);
		let index = cli.owners.len();
		cli.owners.push(owner_name);
		cli.owner_selected = Some(index);
		cli.model_selected = None;
		cli.config.set_owner(cli.owners[index].clone());
		1
	}
}
