/// @description Insert description here
// You can write your code in this editor
event_inherited();
type="text";
verify_function = undefined;

function set_type(new_type,custom_cb){
	type=new_type;
	var _fn;
	switch (new_type) {
		case "text":
		case "password":
		case "email":
			_fn = undefined;
		break;
		case "real":
		case "float":
		case "double":
		case "decimal":
			_fn = verify_real;
		break;
		case "int":
		case "integer":
			_fn = verify_int;
		break;
		case "custom":
			_fn = custom_cb;
		break;
		default:
			_fn = undefined;
		break;
	}
	if (_fn != undefined) {
		_fn = method(id,_fn);
	}
	verify_function = _fn;
}

maskchar = "•";
w = 0;
lerp_speed = 0.25;
label = "";
//*
// Custom callback functions accept one argument
//  - Returns true if the value passed satisfies all checks
//  - Returns false otherwise
//*