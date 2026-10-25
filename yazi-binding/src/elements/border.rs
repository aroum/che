use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, Table, UserData, Value};
use ratatui::widgets::{Borders, Widget};

use super::{Area, BorderSet, BorderType, Edge};
use crate::elements::Line;

#[derive(Clone, Debug, Default)]
pub struct Border {
	pub area: Area,

	pub edge:  Edge,
	pub set:   BorderSet,
	pub style: ratatui::style::Style,

	pub titles: Vec<(ratatui::widgets::TitlePosition, ratatui::text::Line<'static>)>,
}

impl Border {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, edge): (Table, Edge)| {
			Ok(Self { edge, set: BorderType::Rounded.into(), ..Default::default() })
		})?;

		let border = BorderType::compose(lua)?;

		border.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;
		border.into_lua(lua)
	}
}

impl Widget for Border {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		let mut block = ratatui::widgets::Block::default()
			.borders(self.edge.0)
			.border_set(self.set.as_set())
			.border_style(self.style);

		for title in self.titles {
			block = match title {
				(ratatui::widgets::TitlePosition::Top, line) => block.title_top(line),
				(ratatui::widgets::TitlePosition::Bottom, line) => block.title_bottom(line),
			};
		}

		block.render(rect, buf);
	}
}

impl Widget for &Border {
	fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer)
	where
		Self: Sized,
	{
		self.clone().render(rect, buf);
	}
}

impl UserData for Border {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("type", |_, (ud, value): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.set = BorderType::from(value).into();
			Ok(ud)
		});
		methods.add_function_mut("set", |_, (ud, value): (AnyUserData, BorderSet)| {
			ud.borrow_mut::<Self>()?.set = value;
			Ok(ud)
		});
		methods.add_function_mut(
			"title",
			|_, (ud, line, position): (AnyUserData, Line, Option<u8>)| {
				let position = if position == Some(Borders::BOTTOM.bits()) {
					ratatui::widgets::TitlePosition::Bottom
				} else {
					ratatui::widgets::TitlePosition::Top
				};

				ud.borrow_mut::<Self>()?.titles.push((position, line.inner));
				Ok(ud)
			},
		);
		methods.add_function_mut("edge", |_, (ud, edge): (AnyUserData, Edge)| {
			ud.borrow_mut::<Self>()?.edge = edge;
			Ok(ud)
		});
	}
}
