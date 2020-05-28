/// @description Insert description here

if (focused) {
	if (keyboard_check(vk_tab)) {
		io_clear();
		
		var _tin = tab_index + 1, _found = false, _low = _tin, _low_id = noone;
		with (par_control) {
			_low = tab_index < _low ? tab_index : _low;
			if (_low == tab_index) {
				_low_id = id;	
			}
			if (tab_index == _tin) {
				with (par_control) {
					focused = false;	
				}
				focused = true;
				_found = true;
				break;
			}
		}
		if (_found == false) {
			with (par_control) {
				focused = false;	
			}
			with (_low_id) {
				focused = true;	
			}
		}
	}
}
