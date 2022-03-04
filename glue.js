
// Bindings utilities

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function WrapperObject() {
}
WrapperObject.prototype = Object.create(WrapperObject.prototype);
WrapperObject.prototype.constructor = WrapperObject;
WrapperObject.prototype.__class__ = WrapperObject;
WrapperObject.__cache__ = {};
Module['WrapperObject'] = WrapperObject;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant)
    @param {*=} __class__ */
function getCache(__class__) {
  return (__class__ || WrapperObject).__cache__;
}
Module['getCache'] = getCache;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant)
    @param {*=} __class__ */
function wrapPointer(ptr, __class__) {
  var cache = getCache(__class__);
  var ret = cache[ptr];
  if (ret) return ret;
  ret = Object.create((__class__ || WrapperObject).prototype);
  ret.ptr = ptr;
  return cache[ptr] = ret;
}
Module['wrapPointer'] = wrapPointer;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function castObject(obj, __class__) {
  return wrapPointer(obj.ptr, __class__);
}
Module['castObject'] = castObject;

Module['NULL'] = wrapPointer(0);

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function destroy(obj) {
  if (!obj['__destroy__']) throw 'Error: Cannot destroy object. (Did you create it yourself?)';
  obj['__destroy__']();
  // Remove from cache, so the object can be GC'd and refs added onto it released
  delete getCache(obj.__class__)[obj.ptr];
}
Module['destroy'] = destroy;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function compare(obj1, obj2) {
  return obj1.ptr === obj2.ptr;
}
Module['compare'] = compare;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function getPointer(obj) {
  return obj.ptr;
}
Module['getPointer'] = getPointer;

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function getClass(obj) {
  return obj.__class__;
}
Module['getClass'] = getClass;

// Converts big (string or array) values into a C-style storage, in temporary space

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
var ensureCache = {
  buffer: 0,  // the main buffer of temporary storage
  size: 0,   // the size of buffer
  pos: 0,    // the next free offset in buffer
  temps: [], // extra allocations
  needed: 0, // the total size we need next time

  prepare: function() {
    if (ensureCache.needed) {
      // clear the temps
      for (var i = 0; i < ensureCache.temps.length; i++) {
        Module['_free'](ensureCache.temps[i]);
      }
      ensureCache.temps.length = 0;
      // prepare to allocate a bigger buffer
      Module['_free'](ensureCache.buffer);
      ensureCache.buffer = 0;
      ensureCache.size += ensureCache.needed;
      // clean up
      ensureCache.needed = 0;
    }
    if (!ensureCache.buffer) { // happens first time, or when we need to grow
      ensureCache.size += 128; // heuristic, avoid many small grow events
      ensureCache.buffer = Module['_malloc'](ensureCache.size);
      assert(ensureCache.buffer);
    }
    ensureCache.pos = 0;
  },
  alloc: function(array, view) {
    assert(ensureCache.buffer);
    var bytes = view.BYTES_PER_ELEMENT;
    var len = array.length * bytes;
    len = (len + 7) & -8; // keep things aligned to 8 byte boundaries
    var ret;
    if (ensureCache.pos + len >= ensureCache.size) {
      // we failed to allocate in the buffer, ensureCache time around :(
      assert(len > 0); // null terminator, at least
      ensureCache.needed += len;
      ret = Module['_malloc'](len);
      ensureCache.temps.push(ret);
    } else {
      // we can allocate in the buffer
      ret = ensureCache.buffer + ensureCache.pos;
      ensureCache.pos += len;
    }
    return ret;
  },
  copy: function(array, view, offset) {
    offset >>>= 0;
    var bytes = view.BYTES_PER_ELEMENT;
    switch (bytes) {
      case 2: offset >>>= 1; break;
      case 4: offset >>>= 2; break;
      case 8: offset >>>= 3; break;
    }
    for (var i = 0; i < array.length; i++) {
      view[offset + i] = array[i];
    }
  },
};

/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureString(value) {
  if (typeof value === 'string') {
    var intArray = intArrayFromString(value);
    var offset = ensureCache.alloc(intArray, HEAP8);
    ensureCache.copy(intArray, HEAP8, offset);
    return offset;
  }
  return value;
}
/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureInt8(value) {
  if (typeof value === 'object') {
    var offset = ensureCache.alloc(value, HEAP8);
    ensureCache.copy(value, HEAP8, offset);
    return offset;
  }
  return value;
}
/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureInt16(value) {
  if (typeof value === 'object') {
    var offset = ensureCache.alloc(value, HEAP16);
    ensureCache.copy(value, HEAP16, offset);
    return offset;
  }
  return value;
}
/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureInt32(value) {
  if (typeof value === 'object') {
    var offset = ensureCache.alloc(value, HEAP32);
    ensureCache.copy(value, HEAP32, offset);
    return offset;
  }
  return value;
}
/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureFloat32(value) {
  if (typeof value === 'object') {
    var offset = ensureCache.alloc(value, HEAPF32);
    ensureCache.copy(value, HEAPF32, offset);
    return offset;
  }
  return value;
}
/** @suppress {duplicate} (TODO: avoid emitting this multiple times, it is redundant) */
function ensureFloat64(value) {
  if (typeof value === 'object') {
    var offset = ensureCache.alloc(value, HEAPF64);
    ensureCache.copy(value, HEAPF64, offset);
    return offset;
  }
  return value;
}


// VoidPtr
/** @suppress {undefinedVars, duplicate} @this{Object} */function VoidPtr() { throw "cannot construct a VoidPtr, no constructor in IDL" }
VoidPtr.prototype = Object.create(WrapperObject.prototype);
VoidPtr.prototype.constructor = VoidPtr;
VoidPtr.prototype.__class__ = VoidPtr;
VoidPtr.__cache__ = {};
Module['VoidPtr'] = VoidPtr;

  VoidPtr.prototype['__destroy__'] = VoidPtr.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_VoidPtr___destroy___0(self);
};
// DoublePtr
/** @suppress {undefinedVars, duplicate} @this{Object} */function DoublePtr() { throw "cannot construct a DoublePtr, no constructor in IDL" }
DoublePtr.prototype = Object.create(WrapperObject.prototype);
DoublePtr.prototype.constructor = DoublePtr;
DoublePtr.prototype.__class__ = DoublePtr;
DoublePtr.__cache__ = {};
Module['DoublePtr'] = DoublePtr;

  DoublePtr.prototype['__destroy__'] = DoublePtr.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_DoublePtr___destroy___0(self);
};
// FireSize
/** @suppress {undefinedVars, duplicate} @this{Object} */function FireSize() {
  this.ptr = _emscripten_bind_FireSize_FireSize_0();
  getCache(FireSize)[this.ptr] = this;
};;
FireSize.prototype = Object.create(WrapperObject.prototype);
FireSize.prototype.constructor = FireSize;
FireSize.prototype.__class__ = FireSize;
FireSize.__cache__ = {};
Module['FireSize'] = FireSize;

FireSize.prototype['calculateFireBasicDimensions'] = FireSize.prototype.calculateFireBasicDimensions = /** @suppress {undefinedVars, duplicate} @this{Object} */function(effectiveWindSpeed, windSpeedRateUnits, forwardSpreadRate, spreadRateUnits) {
  var self = this.ptr;
  if (effectiveWindSpeed && typeof effectiveWindSpeed === 'object') effectiveWindSpeed = effectiveWindSpeed.ptr;
  if (windSpeedRateUnits && typeof windSpeedRateUnits === 'object') windSpeedRateUnits = windSpeedRateUnits.ptr;
  if (forwardSpreadRate && typeof forwardSpreadRate === 'object') forwardSpreadRate = forwardSpreadRate.ptr;
  if (spreadRateUnits && typeof spreadRateUnits === 'object') spreadRateUnits = spreadRateUnits.ptr;
  _emscripten_bind_FireSize_calculateFireBasicDimensions_4(self, effectiveWindSpeed, windSpeedRateUnits, forwardSpreadRate, spreadRateUnits);
};;

FireSize.prototype['getFireLengthToWidthRatio'] = FireSize.prototype.getFireLengthToWidthRatio = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_FireSize_getFireLengthToWidthRatio_0(self);
};;

FireSize.prototype['getEccentricity'] = FireSize.prototype.getEccentricity = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_FireSize_getEccentricity_0(self);
};;

FireSize.prototype['getBackingSpreadRate'] = FireSize.prototype.getBackingSpreadRate = /** @suppress {undefinedVars, duplicate} @this{Object} */function(spreadRateUnits) {
  var self = this.ptr;
  if (spreadRateUnits && typeof spreadRateUnits === 'object') spreadRateUnits = spreadRateUnits.ptr;
  return _emscripten_bind_FireSize_getBackingSpreadRate_1(self, spreadRateUnits);
};;

FireSize.prototype['getEllipticalA'] = FireSize.prototype.getEllipticalA = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getEllipticalA_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getEllipticalB'] = FireSize.prototype.getEllipticalB = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getEllipticalB_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getEllipticalC'] = FireSize.prototype.getEllipticalC = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getEllipticalC_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getFireLength'] = FireSize.prototype.getFireLength = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getFireLength_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getMaxFireWidth'] = FireSize.prototype.getMaxFireWidth = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getMaxFireWidth_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getFirePerimeter'] = FireSize.prototype.getFirePerimeter = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getFirePerimeter_3(self, lengthUnits, elapsedTime, timeUnits);
};;

FireSize.prototype['getFireArea'] = FireSize.prototype.getFireArea = /** @suppress {undefinedVars, duplicate} @this{Object} */function(areaUnits, elapsedTime, timeUnits) {
  var self = this.ptr;
  if (areaUnits && typeof areaUnits === 'object') areaUnits = areaUnits.ptr;
  if (elapsedTime && typeof elapsedTime === 'object') elapsedTime = elapsedTime.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_FireSize_getFireArea_3(self, areaUnits, elapsedTime, timeUnits);
};;

  FireSize.prototype['__destroy__'] = FireSize.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_FireSize___destroy___0(self);
};
// SIGContainResource
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContainResource(arrival, production, duration, flank, desc, baseCost, hourCost) {
  ensureCache.prepare();
  if (arrival && typeof arrival === 'object') arrival = arrival.ptr;
  if (production && typeof production === 'object') production = production.ptr;
  if (duration && typeof duration === 'object') duration = duration.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  if (baseCost && typeof baseCost === 'object') baseCost = baseCost.ptr;
  if (hourCost && typeof hourCost === 'object') hourCost = hourCost.ptr;
  if (arrival === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_0(); getCache(SIGContainResource)[this.ptr] = this;return }
  if (production === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_1(arrival); getCache(SIGContainResource)[this.ptr] = this;return }
  if (duration === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_2(arrival, production); getCache(SIGContainResource)[this.ptr] = this;return }
  if (flank === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_3(arrival, production, duration); getCache(SIGContainResource)[this.ptr] = this;return }
  if (desc === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_4(arrival, production, duration, flank); getCache(SIGContainResource)[this.ptr] = this;return }
  if (baseCost === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_5(arrival, production, duration, flank, desc); getCache(SIGContainResource)[this.ptr] = this;return }
  if (hourCost === undefined) { this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_6(arrival, production, duration, flank, desc, baseCost); getCache(SIGContainResource)[this.ptr] = this;return }
  this.ptr = _emscripten_bind_SIGContainResource_SIGContainResource_7(arrival, production, duration, flank, desc, baseCost, hourCost);
  getCache(SIGContainResource)[this.ptr] = this;
};;
SIGContainResource.prototype = Object.create(WrapperObject.prototype);
SIGContainResource.prototype.constructor = SIGContainResource;
SIGContainResource.prototype.__class__ = SIGContainResource;
SIGContainResource.__cache__ = {};
Module['SIGContainResource'] = SIGContainResource;

SIGContainResource.prototype['description'] = SIGContainResource.prototype.description = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return UTF8ToString(_emscripten_bind_SIGContainResource_description_0(self));
};;

SIGContainResource.prototype['print'] = SIGContainResource.prototype.print = /** @suppress {undefinedVars, duplicate} @this{Object} */function(buf, buflen) {
  var self = this.ptr;
  ensureCache.prepare();
  if (buf && typeof buf === 'object') buf = buf.ptr;
  else buf = ensureString(buf);
  if (buflen && typeof buflen === 'object') buflen = buflen.ptr;
  _emscripten_bind_SIGContainResource_print_2(self, buf, buflen);
};;

SIGContainResource.prototype['arrival'] = SIGContainResource.prototype.arrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_arrival_0(self);
};;

SIGContainResource.prototype['hourCost'] = SIGContainResource.prototype.hourCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_hourCost_0(self);
};;

SIGContainResource.prototype['duration'] = SIGContainResource.prototype.duration = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_duration_0(self);
};;

SIGContainResource.prototype['production'] = SIGContainResource.prototype.production = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_production_0(self);
};;

SIGContainResource.prototype['baseCost'] = SIGContainResource.prototype.baseCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_baseCost_0(self);
};;

SIGContainResource.prototype['flank'] = SIGContainResource.prototype.flank = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainResource_flank_0(self);
};;

  SIGContainResource.prototype['__destroy__'] = SIGContainResource.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainResource___destroy___0(self);
};
// SIGContainForce
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContainForce(maxResources) {
  if (maxResources && typeof maxResources === 'object') maxResources = maxResources.ptr;
  this.ptr = _emscripten_bind_SIGContainForce_SIGContainForce_1(maxResources);
  getCache(SIGContainForce)[this.ptr] = this;
};;
SIGContainForce.prototype = Object.create(WrapperObject.prototype);
SIGContainForce.prototype.constructor = SIGContainForce;
SIGContainForce.prototype.__class__ = SIGContainForce;
SIGContainForce.__cache__ = {};
Module['SIGContainForce'] = SIGContainForce;

SIGContainForce.prototype['addResource'] = SIGContainForce.prototype.addResource = /** @suppress {undefinedVars, duplicate} @this{Object} */function(arrival, production, duration, flank, desc, baseCost, hourCost) {
  var self = this.ptr;
  ensureCache.prepare();
  if (arrival && typeof arrival === 'object') arrival = arrival.ptr;
  if (production && typeof production === 'object') production = production.ptr;
  if (duration && typeof duration === 'object') duration = duration.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  if (baseCost && typeof baseCost === 'object') baseCost = baseCost.ptr;
  if (hourCost && typeof hourCost === 'object') hourCost = hourCost.ptr;
  if (production === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_1(self, arrival), SIGContainResource) }
  if (duration === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_2(self, arrival, production), SIGContainResource) }
  if (flank === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_3(self, arrival, production, duration), SIGContainResource) }
  if (desc === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_4(self, arrival, production, duration, flank), SIGContainResource) }
  if (baseCost === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_5(self, arrival, production, duration, flank, desc), SIGContainResource) }
  if (hourCost === undefined) { return wrapPointer(_emscripten_bind_SIGContainForce_addResource_6(self, arrival, production, duration, flank, desc, baseCost), SIGContainResource) }
  return wrapPointer(_emscripten_bind_SIGContainForce_addResource_7(self, arrival, production, duration, flank, desc, baseCost, hourCost), SIGContainResource);
};;

SIGContainForce.prototype['exhausted'] = SIGContainForce.prototype.exhausted = /** @suppress {undefinedVars, duplicate} @this{Object} */function(flank) {
  var self = this.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  return _emscripten_bind_SIGContainForce_exhausted_1(self, flank);
};;

SIGContainForce.prototype['firstArrival'] = SIGContainForce.prototype.firstArrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function(flank) {
  var self = this.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  return _emscripten_bind_SIGContainForce_firstArrival_1(self, flank);
};;

SIGContainForce.prototype['nextArrival'] = SIGContainForce.prototype.nextArrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function(after, until, flank) {
  var self = this.ptr;
  if (after && typeof after === 'object') after = after.ptr;
  if (until && typeof until === 'object') until = until.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  return _emscripten_bind_SIGContainForce_nextArrival_3(self, after, until, flank);
};;

SIGContainForce.prototype['productionRate'] = SIGContainForce.prototype.productionRate = /** @suppress {undefinedVars, duplicate} @this{Object} */function(minutesSinceReport, flank) {
  var self = this.ptr;
  if (minutesSinceReport && typeof minutesSinceReport === 'object') minutesSinceReport = minutesSinceReport.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  return _emscripten_bind_SIGContainForce_productionRate_2(self, minutesSinceReport, flank);
};;

SIGContainForce.prototype['resources'] = SIGContainForce.prototype.resources = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainForce_resources_0(self);
};;

SIGContainForce.prototype['resourceArrival'] = SIGContainForce.prototype.resourceArrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceArrival_1(self, index);
};;

SIGContainForce.prototype['resourceBaseCost'] = SIGContainForce.prototype.resourceBaseCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceBaseCost_1(self, index);
};;

SIGContainForce.prototype['resourceCost'] = SIGContainForce.prototype.resourceCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index, finalTime) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  if (finalTime && typeof finalTime === 'object') finalTime = finalTime.ptr;
  return _emscripten_bind_SIGContainForce_resourceCost_2(self, index, finalTime);
};;

SIGContainForce.prototype['resourceDescription'] = SIGContainForce.prototype.resourceDescription = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return UTF8ToString(_emscripten_bind_SIGContainForce_resourceDescription_1(self, index));
};;

SIGContainForce.prototype['resourceDuration'] = SIGContainForce.prototype.resourceDuration = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceDuration_1(self, index);
};;

SIGContainForce.prototype['resourceFlank'] = SIGContainForce.prototype.resourceFlank = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceFlank_1(self, index);
};;

SIGContainForce.prototype['resourceHourCost'] = SIGContainForce.prototype.resourceHourCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceHourCost_1(self, index);
};;

SIGContainForce.prototype['resourceProduction'] = SIGContainForce.prototype.resourceProduction = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForce_resourceProduction_1(self, index);
};;

  SIGContainForce.prototype['__destroy__'] = SIGContainForce.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainForce___destroy___0(self);
};
// SIGContainForceAdapter
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContainForceAdapter() {
  this.ptr = _emscripten_bind_SIGContainForceAdapter_SIGContainForceAdapter_0();
  getCache(SIGContainForceAdapter)[this.ptr] = this;
};;
SIGContainForceAdapter.prototype = Object.create(WrapperObject.prototype);
SIGContainForceAdapter.prototype.constructor = SIGContainForceAdapter;
SIGContainForceAdapter.prototype.__class__ = SIGContainForceAdapter;
SIGContainForceAdapter.__cache__ = {};
Module['SIGContainForceAdapter'] = SIGContainForceAdapter;

SIGContainForceAdapter.prototype['addResource'] = SIGContainForceAdapter.prototype.addResource = /** @suppress {undefinedVars, duplicate} @this{Object} */function(arrival, production, duration, flank, desc, baseCost, hourCost) {
  var self = this.ptr;
  ensureCache.prepare();
  if (arrival && typeof arrival === 'object') arrival = arrival.ptr;
  if (production && typeof production === 'object') production = production.ptr;
  if (duration && typeof duration === 'object') duration = duration.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  if (baseCost && typeof baseCost === 'object') baseCost = baseCost.ptr;
  if (hourCost && typeof hourCost === 'object') hourCost = hourCost.ptr;
  _emscripten_bind_SIGContainForceAdapter_addResource_7(self, arrival, production, duration, flank, desc, baseCost, hourCost);
};;

SIGContainForceAdapter.prototype['firstArrival'] = SIGContainForceAdapter.prototype.firstArrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function(flank) {
  var self = this.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  return _emscripten_bind_SIGContainForceAdapter_firstArrival_1(self, flank);
};;

SIGContainForceAdapter.prototype['removeResourceAt'] = SIGContainForceAdapter.prototype.removeResourceAt = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainForceAdapter_removeResourceAt_1(self, index);
};;

SIGContainForceAdapter.prototype['removeResourceWithThisDesc'] = SIGContainForceAdapter.prototype.removeResourceWithThisDesc = /** @suppress {undefinedVars, duplicate} @this{Object} */function(desc) {
  var self = this.ptr;
  ensureCache.prepare();
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  return _emscripten_bind_SIGContainForceAdapter_removeResourceWithThisDesc_1(self, desc);
};;

SIGContainForceAdapter.prototype['removeAllResourcesWithThisDesc'] = SIGContainForceAdapter.prototype.removeAllResourcesWithThisDesc = /** @suppress {undefinedVars, duplicate} @this{Object} */function(desc) {
  var self = this.ptr;
  ensureCache.prepare();
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  return _emscripten_bind_SIGContainForceAdapter_removeAllResourcesWithThisDesc_1(self, desc);
};;

  SIGContainForceAdapter.prototype['__destroy__'] = SIGContainForceAdapter.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainForceAdapter___destroy___0(self);
};
// SIGContainSim
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContainSim(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, force, tactic, attackDist, retry, minSteps, maxSteps, maxFireSize, maxFireTime) {
  if (reportSize && typeof reportSize === 'object') reportSize = reportSize.ptr;
  if (reportRate && typeof reportRate === 'object') reportRate = reportRate.ptr;
  if (diurnalROS && typeof diurnalROS === 'object') diurnalROS = diurnalROS.ptr;
  if (fireStartMinutesStartTime && typeof fireStartMinutesStartTime === 'object') fireStartMinutesStartTime = fireStartMinutesStartTime.ptr;
  if (lwRatio && typeof lwRatio === 'object') lwRatio = lwRatio.ptr;
  if (force && typeof force === 'object') force = force.ptr;
  if (tactic && typeof tactic === 'object') tactic = tactic.ptr;
  if (attackDist && typeof attackDist === 'object') attackDist = attackDist.ptr;
  if (retry && typeof retry === 'object') retry = retry.ptr;
  if (minSteps && typeof minSteps === 'object') minSteps = minSteps.ptr;
  if (maxSteps && typeof maxSteps === 'object') maxSteps = maxSteps.ptr;
  if (maxFireSize && typeof maxFireSize === 'object') maxFireSize = maxFireSize.ptr;
  if (maxFireTime && typeof maxFireTime === 'object') maxFireTime = maxFireTime.ptr;
  this.ptr = _emscripten_bind_SIGContainSim_SIGContainSim_13(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, force, tactic, attackDist, retry, minSteps, maxSteps, maxFireSize, maxFireTime);
  getCache(SIGContainSim)[this.ptr] = this;
};;
SIGContainSim.prototype = Object.create(WrapperObject.prototype);
SIGContainSim.prototype.constructor = SIGContainSim;
SIGContainSim.prototype.__class__ = SIGContainSim;
SIGContainSim.__cache__ = {};
Module['SIGContainSim'] = SIGContainSim;

SIGContainSim.prototype['attackDistance'] = SIGContainSim.prototype.attackDistance = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_attackDistance_0(self);
};;

SIGContainSim.prototype['attackPointX'] = SIGContainSim.prototype.attackPointX = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_attackPointX_0(self);
};;

SIGContainSim.prototype['attackPointY'] = SIGContainSim.prototype.attackPointY = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_attackPointY_0(self);
};;

SIGContainSim.prototype['attackTime'] = SIGContainSim.prototype.attackTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_attackTime_0(self);
};;

SIGContainSim.prototype['distanceStep'] = SIGContainSim.prototype.distanceStep = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_distanceStep_0(self);
};;

SIGContainSim.prototype['fireBackAtAttack'] = SIGContainSim.prototype.fireBackAtAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireBackAtAttack_0(self);
};;

SIGContainSim.prototype['fireBackAtReport'] = SIGContainSim.prototype.fireBackAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireBackAtReport_0(self);
};;

SIGContainSim.prototype['fireHeadAtAttack'] = SIGContainSim.prototype.fireHeadAtAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireHeadAtAttack_0(self);
};;

SIGContainSim.prototype['fireHeadAtReport'] = SIGContainSim.prototype.fireHeadAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireHeadAtReport_0(self);
};;

SIGContainSim.prototype['fireLwRatioAtReport'] = SIGContainSim.prototype.fireLwRatioAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireLwRatioAtReport_0(self);
};;

SIGContainSim.prototype['fireReportTime'] = SIGContainSim.prototype.fireReportTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireReportTime_0(self);
};;

SIGContainSim.prototype['fireSizeAtReport'] = SIGContainSim.prototype.fireSizeAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireSizeAtReport_0(self);
};;

SIGContainSim.prototype['fireSpreadRateAtBack'] = SIGContainSim.prototype.fireSpreadRateAtBack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireSpreadRateAtBack_0(self);
};;

SIGContainSim.prototype['fireSpreadRateAtReport'] = SIGContainSim.prototype.fireSpreadRateAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_fireSpreadRateAtReport_0(self);
};;

SIGContainSim.prototype['force'] = SIGContainSim.prototype.force = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return wrapPointer(_emscripten_bind_SIGContainSim_force_0(self), SIGContainForce);
};;

SIGContainSim.prototype['maximumSimulationSteps'] = SIGContainSim.prototype.maximumSimulationSteps = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_maximumSimulationSteps_0(self);
};;

SIGContainSim.prototype['minimumSimulationSteps'] = SIGContainSim.prototype.minimumSimulationSteps = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_minimumSimulationSteps_0(self);
};;

SIGContainSim.prototype['status'] = SIGContainSim.prototype.status = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_status_0(self);
};;

SIGContainSim.prototype['tactic'] = SIGContainSim.prototype.tactic = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_tactic_0(self);
};;

SIGContainSim.prototype['finalFireCost'] = SIGContainSim.prototype.finalFireCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFireCost_0(self);
};;

SIGContainSim.prototype['finalFireLine'] = SIGContainSim.prototype.finalFireLine = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFireLine_0(self);
};;

SIGContainSim.prototype['finalFirePerimeter'] = SIGContainSim.prototype.finalFirePerimeter = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFirePerimeter_0(self);
};;

SIGContainSim.prototype['finalFireSize'] = SIGContainSim.prototype.finalFireSize = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFireSize_0(self);
};;

SIGContainSim.prototype['finalFireSweep'] = SIGContainSim.prototype.finalFireSweep = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFireSweep_0(self);
};;

SIGContainSim.prototype['finalFireTime'] = SIGContainSim.prototype.finalFireTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalFireTime_0(self);
};;

SIGContainSim.prototype['finalResourcesUsed'] = SIGContainSim.prototype.finalResourcesUsed = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_finalResourcesUsed_0(self);
};;

SIGContainSim.prototype['fireHeadX'] = SIGContainSim.prototype.fireHeadX = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return wrapPointer(_emscripten_bind_SIGContainSim_fireHeadX_0(self), DoublePtr);
};;

SIGContainSim.prototype['firePerimeterY'] = SIGContainSim.prototype.firePerimeterY = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return wrapPointer(_emscripten_bind_SIGContainSim_firePerimeterY_0(self), DoublePtr);
};;

SIGContainSim.prototype['firePerimeterX'] = SIGContainSim.prototype.firePerimeterX = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return wrapPointer(_emscripten_bind_SIGContainSim_firePerimeterX_0(self), DoublePtr);
};;

SIGContainSim.prototype['firePoints'] = SIGContainSim.prototype.firePoints = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainSim_firePoints_0(self);
};;

SIGContainSim.prototype['checkmem'] = SIGContainSim.prototype.checkmem = /** @suppress {undefinedVars, duplicate} @this{Object} */function(fileName, lineNumber, ptr, type, size) {
  var self = this.ptr;
  ensureCache.prepare();
  if (fileName && typeof fileName === 'object') fileName = fileName.ptr;
  else fileName = ensureString(fileName);
  if (lineNumber && typeof lineNumber === 'object') lineNumber = lineNumber.ptr;
  if (ptr && typeof ptr === 'object') ptr = ptr.ptr;
  if (type && typeof type === 'object') type = type.ptr;
  else type = ensureString(type);
  if (size && typeof size === 'object') size = size.ptr;
  _emscripten_bind_SIGContainSim_checkmem_5(self, fileName, lineNumber, ptr, type, size);
};;

SIGContainSim.prototype['run'] = SIGContainSim.prototype.run = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainSim_run_0(self);
};;

SIGContainSim.prototype['UncontainedArea'] = SIGContainSim.prototype.UncontainedArea = /** @suppress {undefinedVars, duplicate} @this{Object} */function(head, lwRatio, x, y, tactic) {
  var self = this.ptr;
  if (head && typeof head === 'object') head = head.ptr;
  if (lwRatio && typeof lwRatio === 'object') lwRatio = lwRatio.ptr;
  if (x && typeof x === 'object') x = x.ptr;
  if (y && typeof y === 'object') y = y.ptr;
  if (tactic && typeof tactic === 'object') tactic = tactic.ptr;
  return _emscripten_bind_SIGContainSim_UncontainedArea_5(self, head, lwRatio, x, y, tactic);
};;

  SIGContainSim.prototype['__destroy__'] = SIGContainSim.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainSim___destroy___0(self);
};
// SIGDiurnalROS
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGDiurnalROS() {
  this.ptr = _emscripten_bind_SIGDiurnalROS_SIGDiurnalROS_0();
  getCache(SIGDiurnalROS)[this.ptr] = this;
};;
SIGDiurnalROS.prototype = Object.create(WrapperObject.prototype);
SIGDiurnalROS.prototype.constructor = SIGDiurnalROS;
SIGDiurnalROS.prototype.__class__ = SIGDiurnalROS;
SIGDiurnalROS.__cache__ = {};
Module['SIGDiurnalROS'] = SIGDiurnalROS;

SIGDiurnalROS.prototype['push'] = SIGDiurnalROS.prototype.push = /** @suppress {undefinedVars, duplicate} @this{Object} */function(v) {
  var self = this.ptr;
  if (v && typeof v === 'object') v = v.ptr;
  _emscripten_bind_SIGDiurnalROS_push_1(self, v);
};;

SIGDiurnalROS.prototype['at'] = SIGDiurnalROS.prototype.at = /** @suppress {undefinedVars, duplicate} @this{Object} */function(i) {
  var self = this.ptr;
  if (i && typeof i === 'object') i = i.ptr;
  return _emscripten_bind_SIGDiurnalROS_at_1(self, i);
};;

SIGDiurnalROS.prototype['size'] = SIGDiurnalROS.prototype.size = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGDiurnalROS_size_0(self);
};;

  SIGDiurnalROS.prototype['__destroy__'] = SIGDiurnalROS.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGDiurnalROS___destroy___0(self);
};
// SIGContain
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContain(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, distStep, flank, force, attackTime, tactic, attackDist) {
  if (reportSize && typeof reportSize === 'object') reportSize = reportSize.ptr;
  if (reportRate && typeof reportRate === 'object') reportRate = reportRate.ptr;
  if (diurnalROS && typeof diurnalROS === 'object') diurnalROS = diurnalROS.ptr;
  if (fireStartMinutesStartTime && typeof fireStartMinutesStartTime === 'object') fireStartMinutesStartTime = fireStartMinutesStartTime.ptr;
  if (lwRatio && typeof lwRatio === 'object') lwRatio = lwRatio.ptr;
  if (distStep && typeof distStep === 'object') distStep = distStep.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  if (force && typeof force === 'object') force = force.ptr;
  if (attackTime && typeof attackTime === 'object') attackTime = attackTime.ptr;
  if (tactic && typeof tactic === 'object') tactic = tactic.ptr;
  if (attackDist && typeof attackDist === 'object') attackDist = attackDist.ptr;
  this.ptr = _emscripten_bind_SIGContain_SIGContain_11(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, distStep, flank, force, attackTime, tactic, attackDist);
  getCache(SIGContain)[this.ptr] = this;
};;
SIGContain.prototype = Object.create(WrapperObject.prototype);
SIGContain.prototype.constructor = SIGContain;
SIGContain.prototype.__class__ = SIGContain;
SIGContain.__cache__ = {};
Module['SIGContain'] = SIGContain;

SIGContain.prototype['simulationTime'] = SIGContain.prototype.simulationTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_simulationTime_0(self);
};;

SIGContain.prototype['fireSpreadRateAtBack'] = SIGContain.prototype.fireSpreadRateAtBack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireSpreadRateAtBack_0(self);
};;

SIGContain.prototype['fireLwRatioAtReport'] = SIGContain.prototype.fireLwRatioAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireLwRatioAtReport_0(self);
};;

SIGContain.prototype['force'] = SIGContain.prototype.force = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return wrapPointer(_emscripten_bind_SIGContain_force_0(self), SIGContainForce);
};;

SIGContain.prototype['resourceHourCost'] = SIGContain.prototype.resourceHourCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContain_resourceHourCost_1(self, index);
};;

SIGContain.prototype['attackDistance'] = SIGContain.prototype.attackDistance = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_attackDistance_0(self);
};;

SIGContain.prototype['attackPointX'] = SIGContain.prototype.attackPointX = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_attackPointX_0(self);
};;

SIGContain.prototype['fireHeadAtAttack'] = SIGContain.prototype.fireHeadAtAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireHeadAtAttack_0(self);
};;

SIGContain.prototype['attackPointY'] = SIGContain.prototype.attackPointY = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_attackPointY_0(self);
};;

SIGContain.prototype['attackTime'] = SIGContain.prototype.attackTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_attackTime_0(self);
};;

SIGContain.prototype['resourceBaseCost'] = SIGContain.prototype.resourceBaseCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContain_resourceBaseCost_1(self, index);
};;

SIGContain.prototype['fireSpreadRateAtReport'] = SIGContain.prototype.fireSpreadRateAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireSpreadRateAtReport_0(self);
};;

SIGContain.prototype['fireHeadAtReport'] = SIGContain.prototype.fireHeadAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireHeadAtReport_0(self);
};;

SIGContain.prototype['fireReportTime'] = SIGContain.prototype.fireReportTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireReportTime_0(self);
};;

SIGContain.prototype['resourceProduction'] = SIGContain.prototype.resourceProduction = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContain_resourceProduction_1(self, index);
};;

SIGContain.prototype['fireBackAtAttack'] = SIGContain.prototype.fireBackAtAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireBackAtAttack_0(self);
};;

SIGContain.prototype['simulationStep'] = SIGContain.prototype.simulationStep = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_simulationStep_0(self);
};;

SIGContain.prototype['tactic'] = SIGContain.prototype.tactic = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_tactic_0(self);
};;

SIGContain.prototype['resourceDescription'] = SIGContain.prototype.resourceDescription = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return UTF8ToString(_emscripten_bind_SIGContain_resourceDescription_1(self, index));
};;

SIGContain.prototype['distanceStep'] = SIGContain.prototype.distanceStep = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_distanceStep_0(self);
};;

SIGContain.prototype['status'] = SIGContain.prototype.status = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_status_0(self);
};;

SIGContain.prototype['resourceArrival'] = SIGContain.prototype.resourceArrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContain_resourceArrival_1(self, index);
};;

SIGContain.prototype['fireSizeAtReport'] = SIGContain.prototype.fireSizeAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireSizeAtReport_0(self);
};;

SIGContain.prototype['setFireStartTimeMinutes'] = SIGContain.prototype.setFireStartTimeMinutes = /** @suppress {undefinedVars, duplicate} @this{Object} */function(starttime) {
  var self = this.ptr;
  if (starttime && typeof starttime === 'object') starttime = starttime.ptr;
  return _emscripten_bind_SIGContain_setFireStartTimeMinutes_1(self, starttime);
};;

SIGContain.prototype['fireBackAtReport'] = SIGContain.prototype.fireBackAtReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_fireBackAtReport_0(self);
};;

SIGContain.prototype['resourceDuration'] = SIGContain.prototype.resourceDuration = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContain_resourceDuration_1(self, index);
};;

SIGContain.prototype['resources'] = SIGContain.prototype.resources = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_resources_0(self);
};;

SIGContain.prototype['exhaustedTime'] = SIGContain.prototype.exhaustedTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContain_exhaustedTime_0(self);
};;

  SIGContain.prototype['__destroy__'] = SIGContain.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContain___destroy___0(self);
};
// SIGContainAdapter

SIGContainAdapter.prototype['setReportSize'] = SIGContainAdapter.prototype.setReportSize = /** @suppress {undefinedVars, duplicate} @this{Object} */function(reportSize, areaUnits) {
  var self = this.ptr;
  if (reportSize && typeof reportSize === 'object') reportSize = reportSize.ptr;
  if (areaUnits && typeof areaUnits === 'object') areaUnits = areaUnits.ptr;
  _emscripten_bind_SIGContainAdapter_setReportSize_2(self, reportSize, areaUnits);
};;

SIGContainAdapter.prototype['setLwRatio'] = SIGContainAdapter.prototype.setLwRatio = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lwRatio) {
  var self = this.ptr;
  if (lwRatio && typeof lwRatio === 'object') lwRatio = lwRatio.ptr;
  _emscripten_bind_SIGContainAdapter_setLwRatio_1(self, lwRatio);
};;

SIGContainAdapter.prototype['setMaxFireTime'] = SIGContainAdapter.prototype.setMaxFireTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function(maxFireTime) {
  var self = this.ptr;
  if (maxFireTime && typeof maxFireTime === 'object') maxFireTime = maxFireTime.ptr;
  _emscripten_bind_SIGContainAdapter_setMaxFireTime_1(self, maxFireTime);
};;

SIGContainAdapter.prototype['doContainRun'] = SIGContainAdapter.prototype.doContainRun = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainAdapter_doContainRun_0(self);
};;

SIGContainAdapter.prototype['setReportRate'] = SIGContainAdapter.prototype.setReportRate = /** @suppress {undefinedVars, duplicate} @this{Object} */function(reportRate, speedUnits) {
  var self = this.ptr;
  if (reportRate && typeof reportRate === 'object') reportRate = reportRate.ptr;
  if (speedUnits && typeof speedUnits === 'object') speedUnits = speedUnits.ptr;
  _emscripten_bind_SIGContainAdapter_setReportRate_2(self, reportRate, speedUnits);
};;

SIGContainAdapter.prototype['getPerimiterAtInitialAttack'] = SIGContainAdapter.prototype.getPerimiterAtInitialAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getPerimiterAtInitialAttack_1(self, lengthUnits);
};;

SIGContainAdapter.prototype['setTactic'] = SIGContainAdapter.prototype.setTactic = /** @suppress {undefinedVars, duplicate} @this{Object} */function(tactic) {
  var self = this.ptr;
  if (tactic && typeof tactic === 'object') tactic = tactic.ptr;
  _emscripten_bind_SIGContainAdapter_setTactic_1(self, tactic);
};;

SIGContainAdapter.prototype['removeAllResources'] = SIGContainAdapter.prototype.removeAllResources = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainAdapter_removeAllResources_0(self);
};;

SIGContainAdapter.prototype['getFinalFireSize'] = SIGContainAdapter.prototype.getFinalFireSize = /** @suppress {undefinedVars, duplicate} @this{Object} */function(areaUnits) {
  var self = this.ptr;
  if (areaUnits && typeof areaUnits === 'object') areaUnits = areaUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getFinalFireSize_1(self, areaUnits);
};;

SIGContainAdapter.prototype['getFireSizeAtInitialAttack'] = SIGContainAdapter.prototype.getFireSizeAtInitialAttack = /** @suppress {undefinedVars, duplicate} @this{Object} */function(areaUnits) {
  var self = this.ptr;
  if (areaUnits && typeof areaUnits === 'object') areaUnits = areaUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getFireSizeAtInitialAttack_1(self, areaUnits);
};;

SIGContainAdapter.prototype['setAttackDistance'] = SIGContainAdapter.prototype.setAttackDistance = /** @suppress {undefinedVars, duplicate} @this{Object} */function(attackDistance, lengthUnits) {
  var self = this.ptr;
  if (attackDistance && typeof attackDistance === 'object') attackDistance = attackDistance.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  _emscripten_bind_SIGContainAdapter_setAttackDistance_2(self, attackDistance, lengthUnits);
};;

SIGContainAdapter.prototype['getContainmentStatus'] = SIGContainAdapter.prototype.getContainmentStatus = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainAdapter_getContainmentStatus_0(self);
};;

SIGContainAdapter.prototype['removeResourceWithThisDesc'] = SIGContainAdapter.prototype.removeResourceWithThisDesc = /** @suppress {undefinedVars, duplicate} @this{Object} */function(desc) {
  var self = this.ptr;
  ensureCache.prepare();
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  return _emscripten_bind_SIGContainAdapter_removeResourceWithThisDesc_1(self, desc);
};;

SIGContainAdapter.prototype['getPerimeterAtContainment'] = SIGContainAdapter.prototype.getPerimeterAtContainment = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getPerimeterAtContainment_1(self, lengthUnits);
};;

SIGContainAdapter.prototype['getFinalFireLineLength'] = SIGContainAdapter.prototype.getFinalFireLineLength = /** @suppress {undefinedVars, duplicate} @this{Object} */function(lengthUnits) {
  var self = this.ptr;
  if (lengthUnits && typeof lengthUnits === 'object') lengthUnits = lengthUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getFinalFireLineLength_1(self, lengthUnits);
};;

SIGContainAdapter.prototype['setRetry'] = SIGContainAdapter.prototype.setRetry = /** @suppress {undefinedVars, duplicate} @this{Object} */function(retry) {
  var self = this.ptr;
  if (retry && typeof retry === 'object') retry = retry.ptr;
  _emscripten_bind_SIGContainAdapter_setRetry_1(self, retry);
};;

SIGContainAdapter.prototype['getFinalContainmentArea'] = SIGContainAdapter.prototype.getFinalContainmentArea = /** @suppress {undefinedVars, duplicate} @this{Object} */function(areaUnits) {
  var self = this.ptr;
  if (areaUnits && typeof areaUnits === 'object') areaUnits = areaUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getFinalContainmentArea_1(self, areaUnits);
};;

SIGContainAdapter.prototype['removeResourceAt'] = SIGContainAdapter.prototype.removeResourceAt = /** @suppress {undefinedVars, duplicate} @this{Object} */function(index) {
  var self = this.ptr;
  if (index && typeof index === 'object') index = index.ptr;
  return _emscripten_bind_SIGContainAdapter_removeResourceAt_1(self, index);
};;

SIGContainAdapter.prototype['removeAllResourcesWithThisDesc'] = SIGContainAdapter.prototype.removeAllResourcesWithThisDesc = /** @suppress {undefinedVars, duplicate} @this{Object} */function(desc) {
  var self = this.ptr;
  ensureCache.prepare();
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  return _emscripten_bind_SIGContainAdapter_removeAllResourcesWithThisDesc_1(self, desc);
};;

SIGContainAdapter.prototype['getFinalCost'] = SIGContainAdapter.prototype.getFinalCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_SIGContainAdapter_getFinalCost_0(self);
};;

SIGContainAdapter.prototype['getFinalTimeSinceReport'] = SIGContainAdapter.prototype.getFinalTimeSinceReport = /** @suppress {undefinedVars, duplicate} @this{Object} */function(timeUnits) {
  var self = this.ptr;
  if (timeUnits && typeof timeUnits === 'object') timeUnits = timeUnits.ptr;
  return _emscripten_bind_SIGContainAdapter_getFinalTimeSinceReport_1(self, timeUnits);
};;

SIGContainAdapter.prototype['setFireStartTime'] = SIGContainAdapter.prototype.setFireStartTime = /** @suppress {undefinedVars, duplicate} @this{Object} */function(fireStartTime) {
  var self = this.ptr;
  if (fireStartTime && typeof fireStartTime === 'object') fireStartTime = fireStartTime.ptr;
  _emscripten_bind_SIGContainAdapter_setFireStartTime_1(self, fireStartTime);
};;

SIGContainAdapter.prototype['setMinSteps'] = SIGContainAdapter.prototype.setMinSteps = /** @suppress {undefinedVars, duplicate} @this{Object} */function(minSteps) {
  var self = this.ptr;
  if (minSteps && typeof minSteps === 'object') minSteps = minSteps.ptr;
  _emscripten_bind_SIGContainAdapter_setMinSteps_1(self, minSteps);
};;

SIGContainAdapter.prototype['setMaxSteps'] = SIGContainAdapter.prototype.setMaxSteps = /** @suppress {undefinedVars, duplicate} @this{Object} */function(maxSteps) {
  var self = this.ptr;
  if (maxSteps && typeof maxSteps === 'object') maxSteps = maxSteps.ptr;
  _emscripten_bind_SIGContainAdapter_setMaxSteps_1(self, maxSteps);
};;
/** @suppress {undefinedVars, duplicate} @this{Object} */function SIGContainAdapter() {
  this.ptr = _emscripten_bind_SIGContainAdapter_SIGContainAdapter_0();
  getCache(SIGContainAdapter)[this.ptr] = this;
};;
SIGContainAdapter.prototype = Object.create(WrapperObject.prototype);
SIGContainAdapter.prototype.constructor = SIGContainAdapter;
SIGContainAdapter.prototype.__class__ = SIGContainAdapter;
SIGContainAdapter.__cache__ = {};
Module['SIGContainAdapter'] = SIGContainAdapter;

SIGContainAdapter.prototype['setMaxFireSize'] = SIGContainAdapter.prototype.setMaxFireSize = /** @suppress {undefinedVars, duplicate} @this{Object} */function(maxFireSize) {
  var self = this.ptr;
  if (maxFireSize && typeof maxFireSize === 'object') maxFireSize = maxFireSize.ptr;
  _emscripten_bind_SIGContainAdapter_setMaxFireSize_1(self, maxFireSize);
};;

SIGContainAdapter.prototype['addResource'] = SIGContainAdapter.prototype.addResource = /** @suppress {undefinedVars, duplicate} @this{Object} */function(arrival, duration, timeUnit, productionRate, productionRateUnits, description, baseCost, hourCost) {
  var self = this.ptr;
  ensureCache.prepare();
  if (arrival && typeof arrival === 'object') arrival = arrival.ptr;
  if (duration && typeof duration === 'object') duration = duration.ptr;
  if (timeUnit && typeof timeUnit === 'object') timeUnit = timeUnit.ptr;
  if (productionRate && typeof productionRate === 'object') productionRate = productionRate.ptr;
  if (productionRateUnits && typeof productionRateUnits === 'object') productionRateUnits = productionRateUnits.ptr;
  if (description && typeof description === 'object') description = description.ptr;
  else description = ensureString(description);
  if (baseCost && typeof baseCost === 'object') baseCost = baseCost.ptr;
  if (hourCost && typeof hourCost === 'object') hourCost = hourCost.ptr;
  _emscripten_bind_SIGContainAdapter_addResource_8(self, arrival, duration, timeUnit, productionRate, productionRateUnits, description, baseCost, hourCost);
};;

  SIGContainAdapter.prototype['__destroy__'] = SIGContainAdapter.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_SIGContainAdapter___destroy___0(self);
};
// PalmettoGallberry

PalmettoGallberry.prototype['initializeMembers'] = PalmettoGallberry.prototype.initializeMembers = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_PalmettoGallberry_initializeMembers_0(self);
};;
/** @suppress {undefinedVars, duplicate} @this{Object} */function PalmettoGallberry() {
  this.ptr = _emscripten_bind_PalmettoGallberry_PalmettoGallberry_0();
  getCache(PalmettoGallberry)[this.ptr] = this;
};;
PalmettoGallberry.prototype = Object.create(WrapperObject.prototype);
PalmettoGallberry.prototype.constructor = PalmettoGallberry;
PalmettoGallberry.prototype.__class__ = PalmettoGallberry;
PalmettoGallberry.__cache__ = {};
Module['PalmettoGallberry'] = PalmettoGallberry;

PalmettoGallberry.prototype['getHeatOfCombustionLive'] = PalmettoGallberry.prototype.getHeatOfCombustionLive = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getHeatOfCombustionLive_0(self);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyLitterLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyLitterLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, overstoryBasalArea) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (overstoryBasalArea && typeof overstoryBasalArea === 'object') overstoryBasalArea = overstoryBasalArea.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLitterLoad_2(self, ageOfRough, overstoryBasalArea);
};;

PalmettoGallberry.prototype['getPalmettoGallberyLiveOneHourLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyLiveOneHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveOneHourLoad_0(self);
};;

PalmettoGallberry.prototype['getPalmettoGallberyDeadFoliageLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyDeadFoliageLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadFoliageLoad_0(self);
};;

PalmettoGallberry.prototype['getHeatOfCombustionDead'] = PalmettoGallberry.prototype.getHeatOfCombustionDead = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getHeatOfCombustionDead_0(self);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyLiveFoliageLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyLiveFoliageLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, palmettoCoverage, heightOfUnderstory) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (palmettoCoverage && typeof palmettoCoverage === 'object') palmettoCoverage = palmettoCoverage.ptr;
  if (heightOfUnderstory && typeof heightOfUnderstory === 'object') heightOfUnderstory = heightOfUnderstory.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveFoliageLoad_3(self, ageOfRough, palmettoCoverage, heightOfUnderstory);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyLiveTenHourLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyLiveTenHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, heightOfUnderstory) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (heightOfUnderstory && typeof heightOfUnderstory === 'object') heightOfUnderstory = heightOfUnderstory.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveTenHourLoad_2(self, ageOfRough, heightOfUnderstory);
};;

PalmettoGallberry.prototype['getPalmettoGallberyDeadTenHourLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyDeadTenHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadTenHourLoad_0(self);
};;

PalmettoGallberry.prototype['getMoistureOfExtinctionDead'] = PalmettoGallberry.prototype.getMoistureOfExtinctionDead = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getMoistureOfExtinctionDead_0(self);
};;

PalmettoGallberry.prototype['getPalmettoGallberyLiveFoliageLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyLiveFoliageLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveFoliageLoad_0(self);
};;

PalmettoGallberry.prototype['getPalmettoGallberyLitterLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyLitterLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyLitterLoad_0(self);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyDeadTenHourLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyDeadTenHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, palmettoCoverage) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (palmettoCoverage && typeof palmettoCoverage === 'object') palmettoCoverage = palmettoCoverage.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadTenHourLoad_2(self, ageOfRough, palmettoCoverage);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyLiveOneHourLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyLiveOneHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, heightOfUnderstory) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (heightOfUnderstory && typeof heightOfUnderstory === 'object') heightOfUnderstory = heightOfUnderstory.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveOneHourLoad_2(self, ageOfRough, heightOfUnderstory);
};;

PalmettoGallberry.prototype['getPalmettoGallberyFuelBedDepth'] = PalmettoGallberry.prototype.getPalmettoGallberyFuelBedDepth = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyFuelBedDepth_0(self);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyDeadFoliageLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyDeadFoliageLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, palmettoCoverage) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (palmettoCoverage && typeof palmettoCoverage === 'object') palmettoCoverage = palmettoCoverage.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadFoliageLoad_2(self, ageOfRough, palmettoCoverage);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyDeadOneHourLoad'] = PalmettoGallberry.prototype.calculatePalmettoGallberyDeadOneHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function(ageOfRough, heightOfUnderstory) {
  var self = this.ptr;
  if (ageOfRough && typeof ageOfRough === 'object') ageOfRough = ageOfRough.ptr;
  if (heightOfUnderstory && typeof heightOfUnderstory === 'object') heightOfUnderstory = heightOfUnderstory.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadOneHourLoad_2(self, ageOfRough, heightOfUnderstory);
};;

PalmettoGallberry.prototype['getPalmettoGallberyLiveTenHourLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyLiveTenHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveTenHourLoad_0(self);
};;

PalmettoGallberry.prototype['getPalmettoGallberyDeadOneHourLoad'] = PalmettoGallberry.prototype.getPalmettoGallberyDeadOneHourLoad = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadOneHourLoad_0(self);
};;

PalmettoGallberry.prototype['calculatePalmettoGallberyFuelBedDepth'] = PalmettoGallberry.prototype.calculatePalmettoGallberyFuelBedDepth = /** @suppress {undefinedVars, duplicate} @this{Object} */function(heightOfUnderstory) {
  var self = this.ptr;
  if (heightOfUnderstory && typeof heightOfUnderstory === 'object') heightOfUnderstory = heightOfUnderstory.ptr;
  return _emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyFuelBedDepth_1(self, heightOfUnderstory);
};;

  PalmettoGallberry.prototype['__destroy__'] = PalmettoGallberry.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_PalmettoGallberry___destroy___0(self);
};
// WesternAspen
/** @suppress {undefinedVars, duplicate} @this{Object} */function WesternAspen() {
  this.ptr = _emscripten_bind_WesternAspen_WesternAspen_0();
  getCache(WesternAspen)[this.ptr] = this;
};;
WesternAspen.prototype = Object.create(WrapperObject.prototype);
WesternAspen.prototype.constructor = WesternAspen;
WesternAspen.prototype.__class__ = WesternAspen;
WesternAspen.__cache__ = {};
Module['WesternAspen'] = WesternAspen;

WesternAspen.prototype['initializeMembers'] = WesternAspen.prototype.initializeMembers = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_WesternAspen_initializeMembers_0(self);
};;

WesternAspen.prototype['calculateAspenMortality'] = WesternAspen.prototype.calculateAspenMortality = /** @suppress {undefinedVars, duplicate} @this{Object} */function(severity, flameLength, DBH) {
  var self = this.ptr;
  if (severity && typeof severity === 'object') severity = severity.ptr;
  if (flameLength && typeof flameLength === 'object') flameLength = flameLength.ptr;
  if (DBH && typeof DBH === 'object') DBH = DBH.ptr;
  return _emscripten_bind_WesternAspen_calculateAspenMortality_3(self, severity, flameLength, DBH);
};;

WesternAspen.prototype['getAspenDBH'] = WesternAspen.prototype.getAspenDBH = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenDBH_0(self);
};;

WesternAspen.prototype['getAspenFuelBedDepth'] = WesternAspen.prototype.getAspenFuelBedDepth = /** @suppress {undefinedVars, duplicate} @this{Object} */function(typeIndex) {
  var self = this.ptr;
  if (typeIndex && typeof typeIndex === 'object') typeIndex = typeIndex.ptr;
  return _emscripten_bind_WesternAspen_getAspenFuelBedDepth_1(self, typeIndex);
};;

WesternAspen.prototype['getAspenHeatOfCombustionDead'] = WesternAspen.prototype.getAspenHeatOfCombustionDead = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenHeatOfCombustionDead_0(self);
};;

WesternAspen.prototype['getAspenHeatOfCombustionLive'] = WesternAspen.prototype.getAspenHeatOfCombustionLive = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenHeatOfCombustionLive_0(self);
};;

WesternAspen.prototype['getAspenLoadDeadOneHour'] = WesternAspen.prototype.getAspenLoadDeadOneHour = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber, aspenCuringLevel) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  if (aspenCuringLevel && typeof aspenCuringLevel === 'object') aspenCuringLevel = aspenCuringLevel.ptr;
  return _emscripten_bind_WesternAspen_getAspenLoadDeadOneHour_2(self, aspenFuelModelNumber, aspenCuringLevel);
};;

WesternAspen.prototype['getAspenLoadDeadTenHour'] = WesternAspen.prototype.getAspenLoadDeadTenHour = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  return _emscripten_bind_WesternAspen_getAspenLoadDeadTenHour_1(self, aspenFuelModelNumber);
};;

WesternAspen.prototype['getAspenLoadLiveHerbaceous'] = WesternAspen.prototype.getAspenLoadLiveHerbaceous = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber, aspenCuringLevel) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  if (aspenCuringLevel && typeof aspenCuringLevel === 'object') aspenCuringLevel = aspenCuringLevel.ptr;
  return _emscripten_bind_WesternAspen_getAspenLoadLiveHerbaceous_2(self, aspenFuelModelNumber, aspenCuringLevel);
};;

WesternAspen.prototype['getAspenLoadLiveWoody'] = WesternAspen.prototype.getAspenLoadLiveWoody = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber, aspenCuringLevel) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  if (aspenCuringLevel && typeof aspenCuringLevel === 'object') aspenCuringLevel = aspenCuringLevel.ptr;
  return _emscripten_bind_WesternAspen_getAspenLoadLiveWoody_2(self, aspenFuelModelNumber, aspenCuringLevel);
};;

WesternAspen.prototype['getAspenMoistureOfExtinctionDead'] = WesternAspen.prototype.getAspenMoistureOfExtinctionDead = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenMoistureOfExtinctionDead_0(self);
};;

WesternAspen.prototype['getAspenMortality'] = WesternAspen.prototype.getAspenMortality = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenMortality_0(self);
};;

WesternAspen.prototype['getAspenSavrDeadOneHour'] = WesternAspen.prototype.getAspenSavrDeadOneHour = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber, aspenCuringLevel) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  if (aspenCuringLevel && typeof aspenCuringLevel === 'object') aspenCuringLevel = aspenCuringLevel.ptr;
  return _emscripten_bind_WesternAspen_getAspenSavrDeadOneHour_2(self, aspenFuelModelNumber, aspenCuringLevel);
};;

WesternAspen.prototype['getAspenSavrDeadTenHour'] = WesternAspen.prototype.getAspenSavrDeadTenHour = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenSavrDeadTenHour_0(self);
};;

WesternAspen.prototype['getAspenSavrLiveHerbaceous'] = WesternAspen.prototype.getAspenSavrLiveHerbaceous = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_WesternAspen_getAspenSavrLiveHerbaceous_0(self);
};;

WesternAspen.prototype['getAspenSavrLiveWoody'] = WesternAspen.prototype.getAspenSavrLiveWoody = /** @suppress {undefinedVars, duplicate} @this{Object} */function(aspenFuelModelNumber, aspenCuringLevel) {
  var self = this.ptr;
  if (aspenFuelModelNumber && typeof aspenFuelModelNumber === 'object') aspenFuelModelNumber = aspenFuelModelNumber.ptr;
  if (aspenCuringLevel && typeof aspenCuringLevel === 'object') aspenCuringLevel = aspenCuringLevel.ptr;
  return _emscripten_bind_WesternAspen_getAspenSavrLiveWoody_2(self, aspenFuelModelNumber, aspenCuringLevel);
};;

  WesternAspen.prototype['__destroy__'] = WesternAspen.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_WesternAspen___destroy___0(self);
};
// WindSpeedUtility
/** @suppress {undefinedVars, duplicate} @this{Object} */function WindSpeedUtility() {
  this.ptr = _emscripten_bind_WindSpeedUtility_WindSpeedUtility_0();
  getCache(WindSpeedUtility)[this.ptr] = this;
};;
WindSpeedUtility.prototype = Object.create(WrapperObject.prototype);
WindSpeedUtility.prototype.constructor = WindSpeedUtility;
WindSpeedUtility.prototype.__class__ = WindSpeedUtility;
WindSpeedUtility.__cache__ = {};
Module['WindSpeedUtility'] = WindSpeedUtility;

WindSpeedUtility.prototype['windSpeedAtMidflame'] = WindSpeedUtility.prototype.windSpeedAtMidflame = /** @suppress {undefinedVars, duplicate} @this{Object} */function(windSpeedAtTwentyFeet, windAdjustmentFactor) {
  var self = this.ptr;
  if (windSpeedAtTwentyFeet && typeof windSpeedAtTwentyFeet === 'object') windSpeedAtTwentyFeet = windSpeedAtTwentyFeet.ptr;
  if (windAdjustmentFactor && typeof windAdjustmentFactor === 'object') windAdjustmentFactor = windAdjustmentFactor.ptr;
  return _emscripten_bind_WindSpeedUtility_windSpeedAtMidflame_2(self, windSpeedAtTwentyFeet, windAdjustmentFactor);
};;

WindSpeedUtility.prototype['windSpeedAtTwentyFeetFromTenMeter'] = WindSpeedUtility.prototype.windSpeedAtTwentyFeetFromTenMeter = /** @suppress {undefinedVars, duplicate} @this{Object} */function(windSpeedAtTenMeters) {
  var self = this.ptr;
  if (windSpeedAtTenMeters && typeof windSpeedAtTenMeters === 'object') windSpeedAtTenMeters = windSpeedAtTenMeters.ptr;
  return _emscripten_bind_WindSpeedUtility_windSpeedAtTwentyFeetFromTenMeter_1(self, windSpeedAtTenMeters);
};;

  WindSpeedUtility.prototype['__destroy__'] = WindSpeedUtility.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_WindSpeedUtility___destroy___0(self);
};
(function() {
  function setupEnums() {
    

    // AreaUnits_AreaUnitsEnum

    Module['SquareFeet'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_SquareFeet();

    Module['Acres'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_Acres();

    Module['Hectares'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_Hectares();

    Module['SquareMeters'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_SquareMeters();

    Module['SquareMiles'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_SquareMiles();

    Module['SquareKilometers'] = _emscripten_enum_AreaUnits_AreaUnitsEnum_SquareKilometers();

    

    // LengthUnits_LengthUnitsEnum

    Module['Feet'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Feet();

    Module['Inches'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Inches();

    Module['Centimeters'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Centimeters();

    Module['Meters'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Meters();

    Module['Chains'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Chains();

    Module['Miles'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Miles();

    Module['Kilometers'] = _emscripten_enum_LengthUnits_LengthUnitsEnum_Kilometers();

    

    // LoadingUnits_LoadingUnitsEnum

    Module['PoundsPerSquareFoot'] = _emscripten_enum_LoadingUnits_LoadingUnitsEnum_PoundsPerSquareFoot();

    Module['TonsPerAcre'] = _emscripten_enum_LoadingUnits_LoadingUnitsEnum_TonsPerAcre();

    Module['TonnesPerHectare'] = _emscripten_enum_LoadingUnits_LoadingUnitsEnum_TonnesPerHectare();

    Module['KilogramsPerSquareMeter'] = _emscripten_enum_LoadingUnits_LoadingUnitsEnum_KilogramsPerSquareMeter();

    

    // SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum

    Module['SquareFeetOverCubicFeet'] = _emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareFeetOverCubicFeet();

    Module['SquareMetersOverCubicMeters'] = _emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareMetersOverCubicMeters();

    Module['SquareInchesOverCubicInches'] = _emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareInchesOverCubicInches();

    Module['SquareCentimetersOverCubicCentimers'] = _emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareCentimetersOverCubicCentimers();

    

    // CoverUnits_CoverUnitsEnum

    Module['Fraction'] = _emscripten_enum_CoverUnits_CoverUnitsEnum_Fraction();

    Module['Percent'] = _emscripten_enum_CoverUnits_CoverUnitsEnum_Percent();

    

    // SpeedUnits_SpeedUnitsEnum

    Module['FeetPerMinute'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_FeetPerMinute();

    Module['ChainsPerHour'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_ChainsPerHour();

    Module['MetersPerSecond'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_MetersPerSecond();

    Module['MetersPerMinute'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_MetersPerMinute();

    Module['MilesPerHour'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_MilesPerHour();

    Module['KilometersPerHour'] = _emscripten_enum_SpeedUnits_SpeedUnitsEnum_KilometersPerHour();

    

    // ProbabilityUnits_ProbabilityUnitsEnum

    Module['Fraction'] = _emscripten_enum_ProbabilityUnits_ProbabilityUnitsEnum_Fraction();

    Module['Percent'] = _emscripten_enum_ProbabilityUnits_ProbabilityUnitsEnum_Percent();

    

    // MoistureUnits_MoistureUnitsEnum

    Module['Fraction'] = _emscripten_enum_MoistureUnits_MoistureUnitsEnum_Fraction();

    Module['Percent'] = _emscripten_enum_MoistureUnits_MoistureUnitsEnum_Percent();

    

    // SlopeUnits_SlopeUnitsEnum

    Module['Degrees'] = _emscripten_enum_SlopeUnits_SlopeUnitsEnum_Degrees();

    Module['Percent'] = _emscripten_enum_SlopeUnits_SlopeUnitsEnum_Percent();

    

    // DensityUnits_DensityUnitsEnum

    Module['PoundsPerCubicFoot'] = _emscripten_enum_DensityUnits_DensityUnitsEnum_PoundsPerCubicFoot();

    Module['KilogramsPerCubicMeter'] = _emscripten_enum_DensityUnits_DensityUnitsEnum_KilogramsPerCubicMeter();

    

    // HeatOfCombustionUnits_HeatOfCombustionUnitsEnum

    Module['BtusPerPound'] = _emscripten_enum_HeatOfCombustionUnits_HeatOfCombustionUnitsEnum_BtusPerPound();

    Module['KilojoulesPerKilogram'] = _emscripten_enum_HeatOfCombustionUnits_HeatOfCombustionUnitsEnum_KilojoulesPerKilogram();

    

    // HeatSinkUnits_HeatSinkUnitsEnum

    Module['BtusPerCubicFoot'] = _emscripten_enum_HeatSinkUnits_HeatSinkUnitsEnum_BtusPerCubicFoot();

    Module['KilojoulesPerCubicMeter'] = _emscripten_enum_HeatSinkUnits_HeatSinkUnitsEnum_KilojoulesPerCubicMeter();

    

    // HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum

    Module['BtusPerSquareFoot'] = _emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_BtusPerSquareFoot();

    Module['KilojoulesPerSquareMeterPerSecond'] = _emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_KilojoulesPerSquareMeterPerSecond();

    Module['KilowattsPerSquareMeter'] = _emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_KilowattsPerSquareMeter();

    

    // HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum

    Module['BtusPerSquareFootPerMinute'] = _emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_BtusPerSquareFootPerMinute();

    Module['BtusPerSquareFootPerSecond'] = _emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_BtusPerSquareFootPerSecond();

    Module['KilojoulesPerSquareMeterPerSecond'] = _emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilojoulesPerSquareMeterPerSecond();

    Module['KilojoulesPerSquareMeterPerMinute'] = _emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilojoulesPerSquareMeterPerMinute();

    Module['KilowattsPerSquareMeter'] = _emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilowattsPerSquareMeter();

    

    // FirelineIntensityUnits_FirelineIntensityUnitsEnum

    Module['BtusPerFootPerSecond'] = _emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_BtusPerFootPerSecond();

    Module['BtusPerFootPerMinute'] = _emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_BtusPerFootPerMinute();

    Module['KilojoulesPerMeterPerSecond'] = _emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilojoulesPerMeterPerSecond();

    Module['KilojoulesPerMeterPerMinute'] = _emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilojoulesPerMeterPerMinute();

    Module['KilowattsPerMeter'] = _emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilowattsPerMeter();

    

    // TemperatureUnits_TemperatureUnitsEnum

    Module['Fahrenheit'] = _emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Fahrenheit();

    Module['Celsius'] = _emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Celsius();

    Module['Kelvin'] = _emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Kelvin();

    

    // TimeUnits_TimeUnitsEnum

    Module['Minutes'] = _emscripten_enum_TimeUnits_TimeUnitsEnum_Minutes();

    Module['Seconds'] = _emscripten_enum_TimeUnits_TimeUnitsEnum_Seconds();

    Module['Hours'] = _emscripten_enum_TimeUnits_TimeUnitsEnum_Hours();

    

    // ContainTactic

    Module['HeadAttack'] = _emscripten_enum_ContainTactic_HeadAttack();

    Module['RearAttack'] = _emscripten_enum_ContainTactic_RearAttack();

    

    // ContainStatus

    Module['Unreported'] = _emscripten_enum_ContainStatus_Unreported();

    Module['Reported'] = _emscripten_enum_ContainStatus_Reported();

    Module['Attacked'] = _emscripten_enum_ContainStatus_Attacked();

    Module['Contained'] = _emscripten_enum_ContainStatus_Contained();

    Module['Overrun'] = _emscripten_enum_ContainStatus_Overrun();

    Module['Exhausted'] = _emscripten_enum_ContainStatus_Exhausted();

    Module['Overflow'] = _emscripten_enum_ContainStatus_Overflow();

    Module['SizeLimitExceeded'] = _emscripten_enum_ContainStatus_SizeLimitExceeded();

    Module['TimeLimitExceeded'] = _emscripten_enum_ContainStatus_TimeLimitExceeded();

    

    // ContainFlank

    Module['LeftFlank'] = _emscripten_enum_ContainFlank_LeftFlank();

    Module['RightFlank'] = _emscripten_enum_ContainFlank_RightFlank();

    Module['BothFlanks'] = _emscripten_enum_ContainFlank_BothFlanks();

    Module['NeitherFlank'] = _emscripten_enum_ContainFlank_NeitherFlank();

  }
  if (runtimeInitialized) setupEnums();
  else addOnInit(setupEnums);
})();
