use yazi_macro::{emit, relay};
use yazi_parser::mgr::{DisplaceDoOpt, FilterOpt, FindDoOpt, OpenDoOpt, OpenOpt, SearchOpt, UpdatePeekedOpt, UpdateSpottedOpt};
use yazi_shared::{Id, SStr, UndoOp, url::UrlBuf};

pub struct MgrProxy;

impl MgrProxy {
	pub fn arrow(step: impl Into<SStr>) {
		emit!(Call(relay!(mgr:arrow, [step.into()])));
	}

	pub fn cd(target: impl Into<UrlBuf>) {
		emit!(Call(relay!(mgr:cd, [target.into()]).with("raw", true)));
	}

	pub fn displace_do(tab: Id, opt: DisplaceDoOpt) {
		emit!(Call(relay!(mgr:displace_do).with("tab", tab).with_any("opt", opt)));
	}

	pub fn filter_do(opt: FilterOpt) {
		emit!(Call(relay!(mgr:filter_do).with_any("opt", opt)));
	}

	pub fn find_do(opt: FindDoOpt) {
		emit!(Call(relay!(mgr:find_do).with_any("opt", opt)));
	}

	pub fn open(opt: OpenOpt) {
		emit!(Call(relay!(mgr:open).with_any("opt", opt)));
	}

	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(relay!(mgr:open_do).with_any("opt", opt)));
	}

	pub fn refresh() {
		emit!(Call(relay!(mgr:refresh)));
	}

	pub fn remove_do(targets: Vec<UrlBuf>, permanently: bool) {
		emit!(Call(
			relay!(mgr:remove_do).with("permanently", permanently).with_any("targets", targets)
		));
	}

	pub fn reveal(target: impl Into<UrlBuf>) {
		emit!(Call(relay!(mgr:reveal, [target.into()]).with("raw", true).with("no-dummy", true)));
	}

	pub fn search_do(opt: SearchOpt) {
		emit!(Call(relay!(mgr:search_do).with_any("opt", opt)));
	}

	pub fn tab_rename(tab: Id, name: impl Into<SStr>) {
		emit!(Call(relay!(mgr:tab_rename, [name.into()]).with("tab", tab)));
	}

	pub fn update_paged_by(page: usize, only_if: &UrlBuf) {
		emit!(Call(relay!(mgr:update_paged, [page]).with("only-if", only_if)));
	}

	pub fn update_peeked(opt: UpdatePeekedOpt) {
		emit!(Call(relay!(mgr:update_peeked).with_any("opt", opt)));
	}

	pub fn update_spotted(opt: UpdateSpottedOpt) {
		emit!(Call(relay!(mgr:update_spotted).with_any("opt", opt)));
	}

	pub fn upload<I>(urls: I)
	where
		I: IntoIterator<Item = UrlBuf>,
	{
		emit!(Call(relay!(mgr:upload).with_seq(urls)));
	}

	pub fn undo_push(op: UndoOp) {
		emit!(Call(relay!(mgr:undo_push).with_any("op", op)));
	}

	pub fn undo_push_trash_pair(original: UrlBuf, trash_path: UrlBuf) {
		emit!(Call(
			relay!(mgr:undo_push_trash_pair)
				.with_any("original", original)
				.with_any("trash_path", trash_path)
		));
	}

	pub fn undo_push_copy_pair(from: UrlBuf, to: UrlBuf) {
		emit!(Call(relay!(mgr:undo_push_copy_pair).with_any("from", from).with_any("to", to)));
	}

	pub fn undo_push_move_pair(from: UrlBuf, to: UrlBuf) {
		emit!(Call(relay!(mgr:undo_push_move_pair).with_any("from", from).with_any("to", to)));
	}

	pub fn undo_push_copy_overwritten(original: UrlBuf, trash_path: UrlBuf) {
		emit!(Call(
			relay!(mgr:undo_push_copy_overwritten)
				.with_any("original", original)
				.with_any("trash_path", trash_path)
		));
	}

	pub fn undo_push_move_overwritten(original: UrlBuf, trash_path: UrlBuf) {
		emit!(Call(
			relay!(mgr:undo_push_move_overwritten)
				.with_any("original", original)
				.with_any("trash_path", trash_path)
		));
	}

	pub fn watch() {
		emit!(Call(relay!(mgr:watch)));
	}
}
