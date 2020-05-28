// Script assets have changed for v2.3.0 see
// https://help.yoyogames.com/hc/en-us/articles/360005277377 for more information
function textbox_add(tx,ty,tw,th,type) {
	var _options;
	if (argument_count == 5) {
		_options = {callback:undefined}
	} else {
		_options = argument[5];
	}
	var _txt = instance_create_layer(tx,ty,layer,obj_ui_textbox);
	with (_txt) {
		width = tw;
		height = th;
		if (variable_struct_get(_options,"label") != undefined) {
			label = _options.label;	
		}
		if (type=="custom") {
			var _cb = variable_struct_get(_options,"callback");//_options.callback;
			set_type(type,_cb);	
		} else {
			set_type(type);
		}
	}
	return _txt;
}

function button_add(bx,by,bw,bh,options) {
	var _btn = instance_create_layer(bx,by,layer,obj_ui_button);
	with (_btn) {
		width = bw;
		height = bh;
		var _cb = variable_struct_get(options,"callback");
		var _text = variable_struct_get(options,"text");
		callback = method(id,_cb != undefined ? _cb : function(){});
		instance = other.id;
		text = _text != undefined ? _text : "undefined";
	}
	return _btn;
}

function checkbox_add(cx,cy,cw,ch,options) {
	var _chk = instance_create_layer(cx,cy,layer,obj_ui_checkbox);
	with (_chk) {
		width = cw;
		height = ch;
		checked = variable_struct_get_default(options,"checked",false);
		text = variable_struct_get_default(options,"text","");
		align = variable_struct_get_default(options,"align",UI_ALIGN.RIGHT);
	}
	return _chk;
}

function verify_real(val) {
	try {
		real(val);
		return true;
	} catch(e) {
		return false;
	}
}

function verify_int(val) {
	return (string(val) == string_digits(string(val)));
}

enum UI_ALIGN {
    TOP,
	BOTTOM,
	LEFT,
	RIGHT,
	TOP_LEFT,
	TOP_RIGHT,
	BOTTOM_LEFT,
	BOTTOM_RIGHT,
	
	__SIZE
}

function variable_struct_get_default(struct,variable,def) {
	var val = variable_struct_get(struct,variable);
	if (val == undefined) return def;
	return val;
}