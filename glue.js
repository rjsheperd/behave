
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
// ContainResource
/** @suppress {undefinedVars, duplicate} @this{Object} */function ContainResource(arrival, production, duration, flank, desc, baseCost, hourCost) {
  ensureCache.prepare();
  if (arrival && typeof arrival === 'object') arrival = arrival.ptr;
  if (production && typeof production === 'object') production = production.ptr;
  if (duration && typeof duration === 'object') duration = duration.ptr;
  if (flank && typeof flank === 'object') flank = flank.ptr;
  if (desc && typeof desc === 'object') desc = desc.ptr;
  else desc = ensureString(desc);
  if (baseCost && typeof baseCost === 'object') baseCost = baseCost.ptr;
  if (hourCost && typeof hourCost === 'object') hourCost = hourCost.ptr;
  this.ptr = _emscripten_bind_ContainResource_ContainResource_7(arrival, production, duration, flank, desc, baseCost, hourCost);
  getCache(ContainResource)[this.ptr] = this;
};;
ContainResource.prototype = Object.create(WrapperObject.prototype);
ContainResource.prototype.constructor = ContainResource;
ContainResource.prototype.__class__ = ContainResource;
ContainResource.__cache__ = {};
Module['ContainResource'] = ContainResource;

ContainResource.prototype['description'] = ContainResource.prototype.description = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_description_0(self);
};;

ContainResource.prototype['print'] = ContainResource.prototype.print = /** @suppress {undefinedVars, duplicate} @this{Object} */function(buf, buflen) {
  var self = this.ptr;
  if (buf && typeof buf === 'object') buf = buf.ptr;
  if (buflen && typeof buflen === 'object') buflen = buflen.ptr;
  _emscripten_bind_ContainResource_print_2(self, buf, buflen);
};;

ContainResource.prototype['arrival'] = ContainResource.prototype.arrival = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_arrival_0(self);
};;

ContainResource.prototype['hourCost'] = ContainResource.prototype.hourCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_hourCost_0(self);
};;

ContainResource.prototype['duration'] = ContainResource.prototype.duration = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_duration_0(self);
};;

ContainResource.prototype['production'] = ContainResource.prototype.production = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_production_0(self);
};;

ContainResource.prototype['baseCost'] = ContainResource.prototype.baseCost = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_baseCost_0(self);
};;

ContainResource.prototype['flank'] = ContainResource.prototype.flank = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  return _emscripten_bind_ContainResource_flank_0(self);
};;

  ContainResource.prototype['__destroy__'] = ContainResource.prototype.__destroy__ = /** @suppress {undefinedVars, duplicate} @this{Object} */function() {
  var self = this.ptr;
  _emscripten_bind_ContainResource___destroy___0(self);
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
    

    // Sem_ContainFlank

    Module['LeftFlank'] = _emscripten_enum_Sem_ContainFlank_LeftFlank();

    Module['RightFlank'] = _emscripten_enum_Sem_ContainFlank_RightFlank();

    Module['BothFlanks'] = _emscripten_enum_Sem_ContainFlank_BothFlanks();

    Module['NeitherFlank'] = _emscripten_enum_Sem_ContainFlank_NeitherFlank();

  }
  if (runtimeInitialized) setupEnums();
  else addOnInit(setupEnums);
})();
