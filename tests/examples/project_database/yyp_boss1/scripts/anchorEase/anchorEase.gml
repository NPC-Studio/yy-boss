///Easing(_amount, Ease)
/// @argument startValue
/// @argument endValue
/// @argument time
/// @argument duration
/// @argument easingFunction

function ease(_startValue, _endValue, _time, _duration, _easeFunc) {
	static easeCalculator = new __easeCalculator();
	var _progress = _time / _duration;
	var _easeFunc = easeCalculator.eases[_easeFunc];
	var _ease = _easeFunc(_progress);
	return _startValue + (_endValue - _startValue) * _ease;	
}

function __easeCalculator(){

	enum Ease { 
		Linear,
		SineEaseIn,
		SineEaseOut,
		SineEaseInOut,
		QuadEaseIn,
		QuadEaseOut,
		QuadEaseInOut,
		CubicEaseIn,
		CubicEaseOut,
		CubicEaseInOut,
		QuartEaseIn,
		QuartEaseOut,
		QuartEaseInOut,
		QuintEaseIn,
		QuintEaseOut,
		QuintEaseInOut,
		ExpoEaseIn,
		ExpoEaseOut,
		ExpoEaseInOut,
		CircEaseIn,
		CircEaseOut,
		CircEaseInOut,
		BackEaseIn,
		BackEaseOut,
		BackEaseInOut,
		ElasticEaseIn,
		ElasticEaseOut,
		ElasticEaseInOut,
		BounceEaseIn,
		BounceEaseOut,
		BounceEaseInOut 
	}
	
	eases = [];
	
	eases[Ease.Linear] = function(_amount) {
		return _amount;
	}
	
	eases[Ease.SineEaseIn] = function(_amount) {
		_amount /= 2;
		return -1 * cos(_amount * pi) + 1;
	}
	
	eases[Ease.SineEaseOut] = function(_amount) {
		_amount /= 4;
	  return sin(_amount * pi * 2);
	}
 
 eases[Ease.SineEaseInOut] = function(_amount) {
		return -0.5 * (cos(_amount * pi) - 1);
	}
	
	eases[Ease.QuadEaseIn] = function(_amount) {
		return _amount * _amount;
	}
	
	eases[Ease.QuadEaseOut] = function(_amount) {
		return -1 * _amount * (_amount - 2);
	}
	
	eases[Ease.ElasticEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1) return 0.5 * _amount * _amount;
	  _amount -= 1
	  return -0.5 * (_amount * (_amount - 2) - 1);
	}
 
	eases[Ease.CubicEaseIn] = function(_amount) {
		return _amount * _amount * _amount;
	}
	
	eases[Ease.CubicEaseOut] = function(_amount) {
		_amount -= 1;
	  return _amount * _amount * _amount + 1;
	}
	
	eases[Ease.CubicEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1)
	    return 0.5 * _amount * _amount * _amount;
	  _amount -= 2;
	  return 0.5 * (_amount * _amount * _amount + 2);
	}
	
	eases[Ease.QuartEaseIn] = function(_amount) {
		return _amount * _amount * _amount * _amount;
	}
	
	eases[Ease.QuartEaseOut] = function(_amount) {
		_amount -= 1;
	  return -(_amount * _amount * _amount * _amount - 1);
	}
	
	eases[Ease.QuartEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1)
	    return 0.5 * _amount * _amount * _amount * _amount;
	  _amount -= 2;
	  return -0.5 * (_amount * _amount * _amount * _amount - 2);
	}
	
	eases[Ease.QuintEaseIn] = function(_amount) {
		return _amount * _amount * _amount * _amount * _amount;
	}
	
	eases[Ease.QuintEaseOut] = function(_amount) {
		_amount -= 1;
	  return _amount * _amount * _amount * _amount * _amount + 1;
	}
	
	eases[Ease.QuintEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1)
	    return 0.5 * _amount * _amount * _amount * _amount * _amount;
	  _amount -= 2;
	  return 0.5 * (_amount * _amount * _amount * _amount * _amount + 2);
	}
	
	eases[Ease.ExpoEaseIn] = function(_amount) {
		return power(2, 10 * (_amount - 1));
	}
	
	eases[Ease.ExpoEaseOut] = function(_amount) {
		return -power(2, -10 * _amount) + 1;
	}
	
	eases[Ease.ExpoEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1)
	    return 0.5 * power(2, 10 * (_amount - 1));
	  return 0.5 * (-power(2, -10 * (_amount - 1)) + 2);
	}
	
	eases[Ease.CircEaseIn] = function(_amount) {
		return -1 * (sqrt(1 - _amount * _amount) - 1);
	}
	
	eases[Ease.CircEaseOut] = function(_amount) {
		_amount -= 1;
	  return sqrt(1 - _amount * _amount);
	}
	
	eases[Ease.CircEaseInOut] = function(_amount) {
		_amount *= 2;
	  if (_amount < 1)
	    return -0.5 * (sqrt(1 - _amount * _amount) - 1);
	  _amount -= 2;
	  return 0.5 * (sqrt(1 - _amount * _amount) + 1);
	}
	
	eases[Ease.BackEaseIn] = function(_amount) {
		var overshoot = 1.70158;
	  return _amount * _amount * ((overshoot + 1) * _amount - overshoot);
	}
	
	eases[Ease.BackEaseOut] = function(_amount) {
	  var overshoot = 1.70158;
	  _amount -= 1;
	  return _amount * _amount * ((overshoot + 1) * _amount + overshoot) + 1;
	}
	
	eases[Ease.BackEaseInOut] = function(_amount) {
		var overshoot = 2.5949095;
	  _amount *= 2;
	  if (_amount < 1)
	    return (_amount * _amount * ((overshoot + 1) * _amount - overshoot)) / 2;
	  _amount -= 2;
	  return (_amount * _amount * ((overshoot + 1) * _amount + overshoot)) / 2 + 1;
	}
	
	eases[Ease.ElasticEaseIn] = function(_amount) {
	  var period = 0.3;
	  _amount -= 1;
	  return -power(2, 10 * _amount) * sin((_amount - period / 4) * pi * 2 / period);
	}
	
	eases[Ease.ElasticEaseOut] = function(_amount) {
	  var period = 0.3;
	  return power(2, -10 * _amount) * sin((_amount - period / 4) * pi * 2 / period) + 1;
	}
	
	eases[Ease.ElasticEaseOut] = function(_amount) {
		var period = 0.3;
	  _amount *= 2;
	  _amount -= 1;
	  if (_amount < 0)
	    return -0.5 * power(2, 10 * _amount) * sin((_amount - period / 4) * pi * 2 / period);
	  return power(2, -10 * _amount) * sin((_amount - period / 4) * pi * 2 / period) * 0.5 + 1;
	}
	
	function __easeBounce(_amount) {
		var _amount = argument[0];
		if (_amount < 1 / 2.75)
		  return 7.5625 * _amount * _amount;
		else if (_amount < 2 / 2.75)
		{
		  _amount -= 1.5 / 2.75;
		  return 7.5625 * _amount * _amount + 0.75;
		}
		else if (_amount < 2.5 / 2.75)
		{
		  _amount -= 2.25 / 2.75;
		  return 7.5625 * _amount * _amount + 0.9375;
		}
		_amount -= 2.625 / 2.75;
		return 7.5625 * _amount * _amount + 0.984375;	
	}
	
	eases[Ease.BounceEaseIn] = function(_amount) {
		return 1 - __easeBounce(1 - _amount);
	}
	
	eases[Ease.BounceEaseOut] = function(_amount) {
		return __easeBounce(_amount);
	}
	
	eases[Ease.BounceEaseInOut] = function(_amount) {
		if (_amount < 0.5) {
	    _amount *= 2;
	    return (1 - __easeBounce(1 - _amount)) * 0.5;
	  }
	  return __easeBounce(_amount * 2 - 1) * 0.5 + 0.5;
	}
}