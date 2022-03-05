
#include <emscripten.h>

extern "C" {

// Not using size_t for array indices as the values used by the javascript code are signed.

EM_JS(void, array_bounds_check_error, (size_t idx, size_t size), {
  throw 'Array index ' + idx + ' out of bounds: [0,' + size + ')';
});

void array_bounds_check(const int array_size, const int array_idx) {
  if (array_idx < 0 || array_idx >= array_size) {
    array_bounds_check_error(array_idx, array_size);
  }
}

// VoidPtr

void EMSCRIPTEN_KEEPALIVE emscripten_bind_VoidPtr___destroy___0(void** self) {
  delete self;
}

// DoublePtr

void EMSCRIPTEN_KEEPALIVE emscripten_bind_DoublePtr___destroy___0(DoublePtr* self) {
  delete self;
}

// FireSize

FireSize* EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_FireSize_0() {
  return new FireSize();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_calculateFireBasicDimensions_4(FireSize* self, double effectiveWindSpeed, SpeedUnits_SpeedUnitsEnum windSpeedRateUnits, double forwardSpreadRate, SpeedUnits_SpeedUnitsEnum spreadRateUnits) {
  self->calculateFireBasicDimensions(effectiveWindSpeed, windSpeedRateUnits, forwardSpreadRate, spreadRateUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getFireLengthToWidthRatio_0(FireSize* self) {
  return self->getFireLengthToWidthRatio();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getEccentricity_0(FireSize* self) {
  return self->getEccentricity();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getBackingSpreadRate_1(FireSize* self, SpeedUnits_SpeedUnitsEnum spreadRateUnits) {
  return self->getBackingSpreadRate(spreadRateUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getEllipticalA_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getEllipticalA(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getEllipticalB_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getEllipticalB(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getEllipticalC_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getEllipticalC(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getFireLength_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getFireLength(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getMaxFireWidth_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getMaxFireWidth(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getFirePerimeter_3(FireSize* self, LengthUnits_LengthUnitsEnum lengthUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getFirePerimeter(lengthUnits, elapsedTime, timeUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize_getFireArea_3(FireSize* self, AreaUnits_AreaUnitsEnum areaUnits, double elapsedTime, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getFireArea(areaUnits, elapsedTime, timeUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_FireSize___destroy___0(FireSize* self) {
  delete self;
}

// SIGContainResource

SIGContainResource* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_SIGContainResource_0() {
  return new SIGContainResource();
}

SIGContainResource* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_SIGContainResource_7(double arrival, double production, double duration, ContainFlank flank, const char* desc, double baseCost, double hourCost) {
  return new SIGContainResource(arrival, production, duration, flank, desc, baseCost, hourCost);
}

const char* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_description_0(SIGContainResource* self) {
  return self->description();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_print_2(SIGContainResource* self, char* buf, int buflen) {
  self->print(buf, buflen);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_arrival_0(SIGContainResource* self) {
  return self->arrival();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_hourCost_0(SIGContainResource* self) {
  return self->hourCost();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_duration_0(SIGContainResource* self) {
  return self->duration();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_production_0(SIGContainResource* self) {
  return self->production();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_baseCost_0(SIGContainResource* self) {
  return self->baseCost();
}

ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource_flank_0(SIGContainResource* self) {
  return self->flank();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainResource___destroy___0(SIGContainResource* self) {
  delete self;
}

// SIGContainForce

SIGContainForce* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_SIGContainForce_1(int maxResources) {
  return new SIGContainForce(maxResources);
}

SIGContainResource* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_addResource_1(SIGContainForce* self, SIGContainResource* arrival) {
  return self->addResource(arrival);
}

SIGContainResource* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_addResource_7(SIGContainForce* self, double arrival, double production, double duration, ContainFlank flank, const char* desc, double baseCost, double hourCost) {
  return self->addResource(arrival, production, duration, flank, desc, baseCost, hourCost);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_exhausted_1(SIGContainForce* self, ContainFlank flank) {
  return self->exhausted(flank);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_firstArrival_1(SIGContainForce* self, ContainFlank flank) {
  return self->firstArrival(flank);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_nextArrival_3(SIGContainForce* self, double after, double until, ContainFlank flank) {
  return self->nextArrival(after, until, flank);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_productionRate_2(SIGContainForce* self, double minutesSinceReport, ContainFlank flank) {
  return self->productionRate(minutesSinceReport, flank);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resources_0(SIGContainForce* self) {
  return self->resources();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceArrival_1(SIGContainForce* self, int index) {
  return self->resourceArrival(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceBaseCost_1(SIGContainForce* self, int index) {
  return self->resourceBaseCost(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceCost_2(SIGContainForce* self, int index, double finalTime) {
  return self->resourceCost(index, finalTime);
}

const char* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceDescription_1(SIGContainForce* self, int index) {
  return self->resourceDescription(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceDuration_1(SIGContainForce* self, int index) {
  return self->resourceDuration(index);
}

ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceFlank_1(SIGContainForce* self, int index) {
  return self->resourceFlank(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceHourCost_1(SIGContainForce* self, int index) {
  return self->resourceHourCost(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce_resourceProduction_1(SIGContainForce* self, int index) {
  return self->resourceProduction(index);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForce___destroy___0(SIGContainForce* self) {
  delete self;
}

// SIGContainForceAdapter

SIGContainForceAdapter* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_SIGContainForceAdapter_0() {
  return new SIGContainForceAdapter();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_addResource_7(SIGContainForceAdapter* self, double arrival, double production, double duration, ContainFlank flank, const char* desc, double baseCost, double hourCost) {
  self->addResource(arrival, production, duration, flank, desc, baseCost, hourCost);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_firstArrival_1(SIGContainForceAdapter* self, ContainFlank flank) {
  return self->firstArrival(flank);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_removeResourceAt_1(SIGContainForceAdapter* self, int index) {
  return self->removeResourceAt(index);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_removeResourceWithThisDesc_1(SIGContainForceAdapter* self, const char* desc) {
  return self->removeResourceWithThisDesc(desc);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter_removeAllResourcesWithThisDesc_1(SIGContainForceAdapter* self, const char* desc) {
  return self->removeAllResourcesWithThisDesc(desc);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainForceAdapter___destroy___0(SIGContainForceAdapter* self) {
  delete self;
}

// SIGContainSim

SIGContainSim* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_SIGContainSim_13(double reportSize, double reportRate, SIGDiurnalROS* diurnalROS, int fireStartMinutesStartTime, double lwRatio, SIGContainForce* force, ContainTactic tactic, double attackDist, bool retry, int minSteps, int maxSteps, int maxFireSize, int maxFireTime) {
  return new SIGContainSim(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, force, tactic, attackDist, retry, minSteps, maxSteps, maxFireSize, maxFireTime);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_attackDistance_0(SIGContainSim* self) {
  return self->attackDistance();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_attackPointX_0(SIGContainSim* self) {
  return self->attackPointX();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_attackPointY_0(SIGContainSim* self) {
  return self->attackPointY();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_attackTime_0(SIGContainSim* self) {
  return self->attackTime();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_distanceStep_0(SIGContainSim* self) {
  return self->distanceStep();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireBackAtAttack_0(SIGContainSim* self) {
  return self->fireBackAtAttack();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireBackAtReport_0(SIGContainSim* self) {
  return self->fireBackAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireHeadAtAttack_0(SIGContainSim* self) {
  return self->fireHeadAtAttack();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireHeadAtReport_0(SIGContainSim* self) {
  return self->fireHeadAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireLwRatioAtReport_0(SIGContainSim* self) {
  return self->fireLwRatioAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireReportTime_0(SIGContainSim* self) {
  return self->fireReportTime();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireSizeAtReport_0(SIGContainSim* self) {
  return self->fireSizeAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireSpreadRateAtBack_0(SIGContainSim* self) {
  return self->fireSpreadRateAtBack();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireSpreadRateAtReport_0(SIGContainSim* self) {
  return self->fireSpreadRateAtReport();
}

SIGContainForce* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_force_0(SIGContainSim* self) {
  return self->force();
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_maximumSimulationSteps_0(SIGContainSim* self) {
  return self->maximumSimulationSteps();
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_minimumSimulationSteps_0(SIGContainSim* self) {
  return self->minimumSimulationSteps();
}

ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_status_0(SIGContainSim* self) {
  return self->status();
}

ContainTactic EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_tactic_0(SIGContainSim* self) {
  return self->tactic();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFireCost_0(SIGContainSim* self) {
  return self->finalFireCost();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFireLine_0(SIGContainSim* self) {
  return self->finalFireLine();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFirePerimeter_0(SIGContainSim* self) {
  return self->finalFirePerimeter();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFireSize_0(SIGContainSim* self) {
  return self->finalFireSize();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFireSweep_0(SIGContainSim* self) {
  return self->finalFireSweep();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalFireTime_0(SIGContainSim* self) {
  return self->finalFireTime();
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_finalResourcesUsed_0(SIGContainSim* self) {
  return self->finalResourcesUsed();
}

DoublePtr* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_fireHeadX_0(SIGContainSim* self) {
  static DoublePtr temp;
  return (temp = self->fireHeadX(), &temp);
}

DoublePtr* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_firePerimeterY_0(SIGContainSim* self) {
  static DoublePtr temp;
  return (temp = self->firePerimeterY(), &temp);
}

DoublePtr* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_firePerimeterX_0(SIGContainSim* self) {
  static DoublePtr temp;
  return (temp = self->firePerimeterX(), &temp);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_firePoints_0(SIGContainSim* self) {
  return self->firePoints();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_checkmem_5(SIGContainSim* self, char* fileName, int lineNumber, void* ptr, char* type, int size) {
  self->checkmem(fileName, lineNumber, ptr, type, size);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_run_0(SIGContainSim* self) {
  self->run();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim_UncontainedArea_5(SIGContainSim* self, double head, double lwRatio, double x, double y, ContainTactic tactic) {
  return self->UncontainedArea(head, lwRatio, x, y, tactic);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainSim___destroy___0(SIGContainSim* self) {
  delete self;
}

// SIGDiurnalROS

SIGDiurnalROS* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGDiurnalROS_SIGDiurnalROS_0() {
  return new SIGDiurnalROS();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGDiurnalROS_push_1(SIGDiurnalROS* self, double v) {
  self->push(v);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGDiurnalROS_at_1(SIGDiurnalROS* self, int i) {
  return self->at(i);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGDiurnalROS_size_0(SIGDiurnalROS* self) {
  return self->size();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGDiurnalROS___destroy___0(SIGDiurnalROS* self) {
  delete self;
}

// SIGContain

SIGContain* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_SIGContain_11(double reportSize, double reportRate, SIGDiurnalROS* diurnalROS, int fireStartMinutesStartTime, double lwRatio, double distStep, ContainFlank flank, SIGContainForce* force, double attackTime, ContainTactic tactic, double attackDist) {
  return new SIGContain(reportSize, reportRate, diurnalROS, fireStartMinutesStartTime, lwRatio, distStep, flank, force, attackTime, tactic, attackDist);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_simulationTime_0(SIGContain* self) {
  return self->simulationTime();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireSpreadRateAtBack_0(SIGContain* self) {
  return self->fireSpreadRateAtBack();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireLwRatioAtReport_0(SIGContain* self) {
  return self->fireLwRatioAtReport();
}

SIGContainForce* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_force_0(SIGContain* self) {
  return self->force();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceHourCost_1(SIGContain* self, int index) {
  return self->resourceHourCost(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_attackDistance_0(SIGContain* self) {
  return self->attackDistance();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_attackPointX_0(SIGContain* self) {
  return self->attackPointX();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireHeadAtAttack_0(SIGContain* self) {
  return self->fireHeadAtAttack();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_attackPointY_0(SIGContain* self) {
  return self->attackPointY();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_attackTime_0(SIGContain* self) {
  return self->attackTime();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceBaseCost_1(SIGContain* self, int index) {
  return self->resourceBaseCost(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireSpreadRateAtReport_0(SIGContain* self) {
  return self->fireSpreadRateAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireHeadAtReport_0(SIGContain* self) {
  return self->fireHeadAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireReportTime_0(SIGContain* self) {
  return self->fireReportTime();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceProduction_1(SIGContain* self, int index) {
  return self->resourceProduction(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireBackAtAttack_0(SIGContain* self) {
  return self->fireBackAtAttack();
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_simulationStep_0(SIGContain* self) {
  return self->simulationStep();
}

ContainTactic EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_tactic_0(SIGContain* self) {
  return self->tactic();
}

const char* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceDescription_1(SIGContain* self, int index) {
  return self->resourceDescription(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_distanceStep_0(SIGContain* self) {
  return self->distanceStep();
}

ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_status_0(SIGContain* self) {
  return self->status();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceArrival_1(SIGContain* self, int index) {
  return self->resourceArrival(index);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireSizeAtReport_0(SIGContain* self) {
  return self->fireSizeAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_setFireStartTimeMinutes_1(SIGContain* self, int starttime) {
  return self->setFireStartTimeMinutes(starttime);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_fireBackAtReport_0(SIGContain* self) {
  return self->fireBackAtReport();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resourceDuration_1(SIGContain* self, int index) {
  return self->resourceDuration(index);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_resources_0(SIGContain* self) {
  return self->resources();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain_exhaustedTime_0(SIGContain* self) {
  return self->exhaustedTime();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContain___destroy___0(SIGContain* self) {
  delete self;
}

// SIGContainAdapter

SIGContainAdapter* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_SIGContainAdapter_0() {
  return new SIGContainAdapter();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setReportSize_2(SIGContainAdapter* self, double reportSize, AreaUnits_AreaUnitsEnum areaUnits) {
  self->setReportSize(reportSize, areaUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setLwRatio_1(SIGContainAdapter* self, double lwRatio) {
  self->setLwRatio(lwRatio);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setMaxFireTime_1(SIGContainAdapter* self, int maxFireTime) {
  self->setMaxFireTime(maxFireTime);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_doContainRun_0(SIGContainAdapter* self) {
  self->doContainRun();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setReportRate_2(SIGContainAdapter* self, double reportRate, SpeedUnits_SpeedUnitsEnum speedUnits) {
  self->setReportRate(reportRate, speedUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getPerimiterAtInitialAttack_1(SIGContainAdapter* self, LengthUnits_LengthUnitsEnum lengthUnits) {
  return self->getPerimiterAtInitialAttack(lengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setTactic_1(SIGContainAdapter* self, ContainTactic tactic) {
  self->setTactic(tactic);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_removeAllResources_0(SIGContainAdapter* self) {
  self->removeAllResources();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFinalFireSize_1(SIGContainAdapter* self, AreaUnits_AreaUnitsEnum areaUnits) {
  return self->getFinalFireSize(areaUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFireSizeAtInitialAttack_1(SIGContainAdapter* self, AreaUnits_AreaUnitsEnum areaUnits) {
  return self->getFireSizeAtInitialAttack(areaUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setAttackDistance_2(SIGContainAdapter* self, double attackDistance, LengthUnits_LengthUnitsEnum lengthUnits) {
  self->setAttackDistance(attackDistance, lengthUnits);
}

ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getContainmentStatus_0(SIGContainAdapter* self) {
  return self->getContainmentStatus();
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_removeResourceWithThisDesc_1(SIGContainAdapter* self, const char* desc) {
  return self->removeResourceWithThisDesc(desc);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getPerimeterAtContainment_1(SIGContainAdapter* self, LengthUnits_LengthUnitsEnum lengthUnits) {
  return self->getPerimeterAtContainment(lengthUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFinalFireLineLength_1(SIGContainAdapter* self, LengthUnits_LengthUnitsEnum lengthUnits) {
  return self->getFinalFireLineLength(lengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setRetry_1(SIGContainAdapter* self, bool retry) {
  self->setRetry(retry);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFinalContainmentArea_1(SIGContainAdapter* self, AreaUnits_AreaUnitsEnum areaUnits) {
  return self->getFinalContainmentArea(areaUnits);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_removeResourceAt_1(SIGContainAdapter* self, int index) {
  return self->removeResourceAt(index);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_removeAllResourcesWithThisDesc_1(SIGContainAdapter* self, const char* desc) {
  return self->removeAllResourcesWithThisDesc(desc);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFinalCost_0(SIGContainAdapter* self) {
  return self->getFinalCost();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_getFinalTimeSinceReport_1(SIGContainAdapter* self, TimeUnits_TimeUnitsEnum timeUnits) {
  return self->getFinalTimeSinceReport(timeUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setFireStartTime_1(SIGContainAdapter* self, int fireStartTime) {
  self->setFireStartTime(fireStartTime);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setMinSteps_1(SIGContainAdapter* self, int minSteps) {
  self->setMinSteps(minSteps);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setMaxSteps_1(SIGContainAdapter* self, int maxSteps) {
  self->setMaxSteps(maxSteps);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_setMaxFireSize_1(SIGContainAdapter* self, int maxFireSize) {
  self->setMaxFireSize(maxFireSize);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter_addResource_8(SIGContainAdapter* self, double arrival, double duration, TimeUnits_TimeUnitsEnum timeUnit, double productionRate, SpeedUnits_SpeedUnitsEnum productionRateUnits, char* description, double baseCost, double hourCost) {
  self->addResource(arrival, duration, timeUnit, productionRate, productionRateUnits, description, baseCost, hourCost);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGContainAdapter___destroy___0(SIGContainAdapter* self) {
  delete self;
}

// SIGSpotInputs

SIGSpotInputs* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_SIGSpotInputs_0() {
  return new SIGSpotInputs();
}

SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getLocation_0(SIGSpotInputs* self) {
  return self->getLocation();
}

SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getTreeSpecies_0(SIGSpotInputs* self) {
  return self->getTreeSpecies();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setBurningPileFlameHeight_2(SIGSpotInputs* self, double buringPileFlameHeight, LengthUnits_LengthUnitsEnum flameHeightUnits) {
  self->setBurningPileFlameHeight(buringPileFlameHeight, flameHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setDBH_2(SIGSpotInputs* self, double DBH, LengthUnits_LengthUnitsEnum DBHUnits) {
  self->setDBH(DBH, DBHUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setDownwindCoverHeight_2(SIGSpotInputs* self, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  self->setDownwindCoverHeight(downwindCoverHeight, coverHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setLocation_1(SIGSpotInputs* self, SpotFireLocation location) {
  self->setLocation(location);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setRidgeToValleyDistance_2(SIGSpotInputs* self, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits) {
  self->setRidgeToValleyDistance(ridgeToValleyDistance, ridgeToValleyDistanceUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setRidgeToValleyElevation_2(SIGSpotInputs* self, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits) {
  self->setRidgeToValleyElevation(ridgeToValleyElevation, elevationUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setSurfaceFlameLength_2(SIGSpotInputs* self, double surfaceFlameLength, LengthUnits_LengthUnitsEnum flameLengthUnits) {
  self->setSurfaceFlameLength(surfaceFlameLength, flameLengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setTorchingTrees_1(SIGSpotInputs* self, int torchingTrees) {
  self->setTorchingTrees(torchingTrees);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setTreeHeight_2(SIGSpotInputs* self, double treeHeight, LengthUnits_LengthUnitsEnum treeHeightUnits) {
  self->setTreeHeight(treeHeight, treeHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setTreeSpecies_1(SIGSpotInputs* self, SpotTreeSpecies treeSpecies) {
  self->setTreeSpecies(treeSpecies);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_setWindSpeedAtTwentyFeet_2(SIGSpotInputs* self, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->setWindSpeedAtTwentyFeet(windSpeedAtTwentyFeet, windSpeedUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_updateSpotInputsForBurningPile_11(SIGSpotInputs* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, double buringPileFlameHeight, LengthUnits_LengthUnitsEnum flameHeightUnits, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->updateSpotInputsForBurningPile(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, buringPileFlameHeight, flameHeightUnits, windSpeedAtTwentyFeet, windSpeedUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_updateSpotInputsForSurfaceFire_11(SIGSpotInputs* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits, double surfaceFlameLength, LengthUnits_LengthUnitsEnum flameLengthUnits) {
  self->updateSpotInputsForSurfaceFire(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, windSpeedAtTwentyFeet, windSpeedUnits, surfaceFlameLength, flameLengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_updateSpotInputsForTorchingTrees_15(SIGSpotInputs* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, int torchingTrees, double DBH, LengthUnits_LengthUnitsEnum DBHUnits, double treeHeight, LengthUnits_LengthUnitsEnum treeHeightUnits, SpotTreeSpecies treeSpecies, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->updateSpotInputsForTorchingTrees(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, torchingTrees, DBH, DBHUnits, treeHeight, treeHeightUnits, treeSpecies, windSpeedAtTwentyFeet, windSpeedUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getBurningPileFlameHeight_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum flameHeightUnits) {
  return self->getBurningPileFlameHeight(flameHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getDBH_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum DBHUnits) {
  return self->getDBH(DBHUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getDownwindCoverHeight_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  return self->getDownwindCoverHeight(coverHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getRidgeToValleyDistance_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits) {
  return self->getRidgeToValleyDistance(ridgeToValleyDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getRidgeToValleyElevation_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum elevationUnits) {
  return self->getRidgeToValleyElevation(elevationUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getSurfaceFlameLength_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum flameLengthUnits) {
  return self->getSurfaceFlameLength(flameLengthUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getTreeHeight_1(SIGSpotInputs* self, LengthUnits_LengthUnitsEnum treeHeightUnits) {
  return self->getTreeHeight(treeHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getWindSpeedAtTwentyFeet_1(SIGSpotInputs* self, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  return self->getWindSpeedAtTwentyFeet(windSpeedUnits);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs_getTorchingTrees_0(SIGSpotInputs* self) {
  return self->getTorchingTrees();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpotInputs___destroy___0(SIGSpotInputs* self) {
  delete self;
}

// SIGSpot

SIGSpot* EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_SIGSpot_0() {
  return new SIGSpot();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_initializeMembers_0(SIGSpot* self) {
  self->initializeMembers();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_calculateSpottingDistanceFromBurningPile_0(SIGSpot* self) {
  self->calculateSpottingDistanceFromBurningPile();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_calculateSpottingDistanceFromSurfaceFire_0(SIGSpot* self) {
  self->calculateSpottingDistanceFromSurfaceFire();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_calculateSpottingDistanceFromTorchingTrees_0(SIGSpot* self) {
  self->calculateSpottingDistanceFromTorchingTrees();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setBurningPileFlameHeight_2(SIGSpot* self, double buringPileflameHeight, LengthUnits_LengthUnitsEnum flameHeightUnits) {
  self->setBurningPileFlameHeight(buringPileflameHeight, flameHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setDBH_2(SIGSpot* self, double DBH, LengthUnits_LengthUnitsEnum DBHUnits) {
  self->setDBH(DBH, DBHUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setDownwindCoverHeight_2(SIGSpot* self, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  self->setDownwindCoverHeight(downwindCoverHeight, coverHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setFlameLength_2(SIGSpot* self, double flameLength, LengthUnits_LengthUnitsEnum flameLengthUnits) {
  self->setFlameLength(flameLength, flameLengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setLocation_1(SIGSpot* self, SpotFireLocation location) {
  self->setLocation(location);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setRidgeToValleyDistance_2(SIGSpot* self, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits) {
  self->setRidgeToValleyDistance(ridgeToValleyDistance, ridgeToValleyDistanceUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setRidgeToValleyElevation_2(SIGSpot* self, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits) {
  self->setRidgeToValleyElevation(ridgeToValleyElevation, elevationUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setTorchingTrees_1(SIGSpot* self, int torchingTrees) {
  self->setTorchingTrees(torchingTrees);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setTreeHeight_2(SIGSpot* self, double treeHeight, LengthUnits_LengthUnitsEnum treeHeightUnits) {
  self->setTreeHeight(treeHeight, treeHeightUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setTreeSpecies_1(SIGSpot* self, SpotTreeSpecies treeSpecies) {
  self->setTreeSpecies(treeSpecies);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_setWindSpeedAtTwentyFeet_2(SIGSpot* self, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->setWindSpeedAtTwentyFeet(windSpeedAtTwentyFeet, windSpeedUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_updateSpotInputsForBurningPile_11(SIGSpot* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, double buringPileFlameHeight, LengthUnits_LengthUnitsEnum flameHeightUnits, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->updateSpotInputsForBurningPile(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, buringPileFlameHeight, flameHeightUnits, windSpeedAtTwentyFeet, windSpeedUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_updateSpotInputsForSurfaceFire_11(SIGSpot* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits, double flameLength, LengthUnits_LengthUnitsEnum flameLengthUnits) {
  self->updateSpotInputsForSurfaceFire(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, windSpeedAtTwentyFeet, windSpeedUnits, flameLength, flameLengthUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_updateSpotInputsForTorchingTrees_15(SIGSpot* self, SpotFireLocation location, double ridgeToValleyDistance, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits, double ridgeToValleyElevation, LengthUnits_LengthUnitsEnum elevationUnits, double downwindCoverHeight, LengthUnits_LengthUnitsEnum coverHeightUnits, int torchingTrees, double DBH, LengthUnits_LengthUnitsEnum DBHUnits, double treeHeight, LengthUnits_LengthUnitsEnum treeHeightUnits, SpotTreeSpecies treeSpecies, double windSpeedAtTwentyFeet, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  self->updateSpotInputsForTorchingTrees(location, ridgeToValleyDistance, ridgeToValleyDistanceUnits, ridgeToValleyElevation, elevationUnits, downwindCoverHeight, coverHeightUnits, torchingTrees, DBH, DBHUnits, treeHeight, treeHeightUnits, treeSpecies, windSpeedAtTwentyFeet, windSpeedUnits);
}

int EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getTorchingTrees_0(SIGSpot* self) {
  return self->getTorchingTrees();
}

SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getLocation_0(SIGSpot* self) {
  return self->getLocation();
}

SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getTreeSpecies_0(SIGSpot* self) {
  return self->getTreeSpecies();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getBurningPileFlameHeight_1(SIGSpot* self, LengthUnits_LengthUnitsEnum flameHeightUnits) {
  return self->getBurningPileFlameHeight(flameHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getCoverHeightUsedForBurningPile_1(SIGSpot* self, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  return self->getCoverHeightUsedForBurningPile(coverHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getCoverHeightUsedForSurfaceFire_1(SIGSpot* self, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  return self->getCoverHeightUsedForSurfaceFire(coverHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getCoverHeightUsedForTorchingTrees_1(SIGSpot* self, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  return self->getCoverHeightUsedForTorchingTrees(coverHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getDBH_1(SIGSpot* self, LengthUnits_LengthUnitsEnum DBHUnits) {
  return self->getDBH(DBHUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getDownwindCoverHeight_1(SIGSpot* self, LengthUnits_LengthUnitsEnum coverHeightUnits) {
  return self->getDownwindCoverHeight(coverHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getFlameDurationForTorchingTrees_1(SIGSpot* self, TimeUnits_TimeUnitsEnum durationUnits) {
  return self->getFlameDurationForTorchingTrees(durationUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getFlameHeightForTorchingTrees_1(SIGSpot* self, LengthUnits_LengthUnitsEnum flameHeightUnits) {
  return self->getFlameHeightForTorchingTrees(flameHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getFlameRatioForTorchingTrees_0(SIGSpot* self) {
  return self->getFlameRatioForTorchingTrees();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFirebrandHeightFromBurningPile_1(SIGSpot* self, LengthUnits_LengthUnitsEnum firebrandHeightUnits) {
  return self->getMaxFirebrandHeightFromBurningPile(firebrandHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFirebrandHeightFromSurfaceFire_1(SIGSpot* self, LengthUnits_LengthUnitsEnum firebrandHeightUnits) {
  return self->getMaxFirebrandHeightFromSurfaceFire(firebrandHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFirebrandHeightFromTorchingTrees_1(SIGSpot* self, LengthUnits_LengthUnitsEnum firebrandHeightUnits) {
  return self->getMaxFirebrandHeightFromTorchingTrees(firebrandHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFlatTerrainSpottingDistanceFromBurningPile_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxFlatTerrainSpottingDistanceFromBurningPile(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFlatTerrainSpottingDistanceFromSurfaceFire_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxFlatTerrainSpottingDistanceFromSurfaceFire(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxFlatTerrainSpottingDistanceFromTorchingTrees_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxFlatTerrainSpottingDistanceFromTorchingTrees(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxMountainousTerrainSpottingDistanceFromBurningPile_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxMountainousTerrainSpottingDistanceFromBurningPile(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxMountainousTerrainSpottingDistanceFromSurfaceFire_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxMountainousTerrainSpottingDistanceFromSurfaceFire(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getMaxMountainousTerrainSpottingDistanceFromTorchingTrees_1(SIGSpot* self, LengthUnits_LengthUnitsEnum spottingDistanceUnits) {
  return self->getMaxMountainousTerrainSpottingDistanceFromTorchingTrees(spottingDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getRidgeToValleyDistance_1(SIGSpot* self, LengthUnits_LengthUnitsEnum ridgeToValleyDistanceUnits) {
  return self->getRidgeToValleyDistance(ridgeToValleyDistanceUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getRidgeToValleyElevation_1(SIGSpot* self, LengthUnits_LengthUnitsEnum elevationUnits) {
  return self->getRidgeToValleyElevation(elevationUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getSurfaceFlameLength_1(SIGSpot* self, LengthUnits_LengthUnitsEnum surfaceFlameLengthUnits) {
  return self->getSurfaceFlameLength(surfaceFlameLengthUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getTreeHeight_1(SIGSpot* self, LengthUnits_LengthUnitsEnum treeHeightUnits) {
  return self->getTreeHeight(treeHeightUnits);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot_getWindSpeedAtTwentyFeet_1(SIGSpot* self, SpeedUnits_SpeedUnitsEnum windSpeedUnits) {
  return self->getWindSpeedAtTwentyFeet(windSpeedUnits);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_SIGSpot___destroy___0(SIGSpot* self) {
  delete self;
}

// PalmettoGallberry

PalmettoGallberry* EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_PalmettoGallberry_0() {
  return new PalmettoGallberry();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_initializeMembers_0(PalmettoGallberry* self) {
  self->initializeMembers();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getHeatOfCombustionLive_0(PalmettoGallberry* self) {
  return self->getHeatOfCombustionLive();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLitterLoad_2(PalmettoGallberry* self, double ageOfRough, double overstoryBasalArea) {
  return self->calculatePalmettoGallberyLitterLoad(ageOfRough, overstoryBasalArea);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveOneHourLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyLiveOneHourLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadFoliageLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyDeadFoliageLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getHeatOfCombustionDead_0(PalmettoGallberry* self) {
  return self->getHeatOfCombustionDead();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveFoliageLoad_3(PalmettoGallberry* self, double ageOfRough, double palmettoCoverage, double heightOfUnderstory) {
  return self->calculatePalmettoGallberyLiveFoliageLoad(ageOfRough, palmettoCoverage, heightOfUnderstory);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveTenHourLoad_2(PalmettoGallberry* self, double ageOfRough, double heightOfUnderstory) {
  return self->calculatePalmettoGallberyLiveTenHourLoad(ageOfRough, heightOfUnderstory);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadTenHourLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyDeadTenHourLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getMoistureOfExtinctionDead_0(PalmettoGallberry* self) {
  return self->getMoistureOfExtinctionDead();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveFoliageLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyLiveFoliageLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyLitterLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyLitterLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadTenHourLoad_2(PalmettoGallberry* self, double ageOfRough, double palmettoCoverage) {
  return self->calculatePalmettoGallberyDeadTenHourLoad(ageOfRough, palmettoCoverage);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyLiveOneHourLoad_2(PalmettoGallberry* self, double ageOfRough, double heightOfUnderstory) {
  return self->calculatePalmettoGallberyLiveOneHourLoad(ageOfRough, heightOfUnderstory);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyFuelBedDepth_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyFuelBedDepth();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadFoliageLoad_2(PalmettoGallberry* self, double ageOfRough, double palmettoCoverage) {
  return self->calculatePalmettoGallberyDeadFoliageLoad(ageOfRough, palmettoCoverage);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyDeadOneHourLoad_2(PalmettoGallberry* self, double ageOfRough, double heightOfUnderstory) {
  return self->calculatePalmettoGallberyDeadOneHourLoad(ageOfRough, heightOfUnderstory);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyLiveTenHourLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyLiveTenHourLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_getPalmettoGallberyDeadOneHourLoad_0(PalmettoGallberry* self) {
  return self->getPalmettoGallberyDeadOneHourLoad();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry_calculatePalmettoGallberyFuelBedDepth_1(PalmettoGallberry* self, double heightOfUnderstory) {
  return self->calculatePalmettoGallberyFuelBedDepth(heightOfUnderstory);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_PalmettoGallberry___destroy___0(PalmettoGallberry* self) {
  delete self;
}

// WesternAspen

WesternAspen* EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_WesternAspen_0() {
  return new WesternAspen();
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_initializeMembers_0(WesternAspen* self) {
  self->initializeMembers();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_calculateAspenMortality_3(WesternAspen* self, int severity, double flameLength, double DBH) {
  return self->calculateAspenMortality(severity, flameLength, DBH);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenDBH_0(WesternAspen* self) {
  return self->getAspenDBH();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenFuelBedDepth_1(WesternAspen* self, int typeIndex) {
  return self->getAspenFuelBedDepth(typeIndex);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenHeatOfCombustionDead_0(WesternAspen* self) {
  return self->getAspenHeatOfCombustionDead();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenHeatOfCombustionLive_0(WesternAspen* self) {
  return self->getAspenHeatOfCombustionLive();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenLoadDeadOneHour_2(WesternAspen* self, int aspenFuelModelNumber, double aspenCuringLevel) {
  return self->getAspenLoadDeadOneHour(aspenFuelModelNumber, aspenCuringLevel);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenLoadDeadTenHour_1(WesternAspen* self, int aspenFuelModelNumber) {
  return self->getAspenLoadDeadTenHour(aspenFuelModelNumber);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenLoadLiveHerbaceous_2(WesternAspen* self, int aspenFuelModelNumber, double aspenCuringLevel) {
  return self->getAspenLoadLiveHerbaceous(aspenFuelModelNumber, aspenCuringLevel);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenLoadLiveWoody_2(WesternAspen* self, int aspenFuelModelNumber, double aspenCuringLevel) {
  return self->getAspenLoadLiveWoody(aspenFuelModelNumber, aspenCuringLevel);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenMoistureOfExtinctionDead_0(WesternAspen* self) {
  return self->getAspenMoistureOfExtinctionDead();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenMortality_0(WesternAspen* self) {
  return self->getAspenMortality();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenSavrDeadOneHour_2(WesternAspen* self, int aspenFuelModelNumber, double aspenCuringLevel) {
  return self->getAspenSavrDeadOneHour(aspenFuelModelNumber, aspenCuringLevel);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenSavrDeadTenHour_0(WesternAspen* self) {
  return self->getAspenSavrDeadTenHour();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenSavrLiveHerbaceous_0(WesternAspen* self) {
  return self->getAspenSavrLiveHerbaceous();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen_getAspenSavrLiveWoody_2(WesternAspen* self, int aspenFuelModelNumber, double aspenCuringLevel) {
  return self->getAspenSavrLiveWoody(aspenFuelModelNumber, aspenCuringLevel);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_WesternAspen___destroy___0(WesternAspen* self) {
  delete self;
}

// WindSpeedUtility

WindSpeedUtility* EMSCRIPTEN_KEEPALIVE emscripten_bind_WindSpeedUtility_WindSpeedUtility_0() {
  return new WindSpeedUtility();
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WindSpeedUtility_windSpeedAtMidflame_2(WindSpeedUtility* self, double windSpeedAtTwentyFeet, double windAdjustmentFactor) {
  return self->windSpeedAtMidflame(windSpeedAtTwentyFeet, windAdjustmentFactor);
}

double EMSCRIPTEN_KEEPALIVE emscripten_bind_WindSpeedUtility_windSpeedAtTwentyFeetFromTenMeter_1(WindSpeedUtility* self, double windSpeedAtTenMeters) {
  return self->windSpeedAtTwentyFeetFromTenMeter(windSpeedAtTenMeters);
}

void EMSCRIPTEN_KEEPALIVE emscripten_bind_WindSpeedUtility___destroy___0(WindSpeedUtility* self) {
  delete self;
}

// AreaUnits_AreaUnitsEnum
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_SquareFeet() {
  return AreaUnits::SquareFeet;
}
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_Acres() {
  return AreaUnits::Acres;
}
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_Hectares() {
  return AreaUnits::Hectares;
}
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_SquareMeters() {
  return AreaUnits::SquareMeters;
}
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_SquareMiles() {
  return AreaUnits::SquareMiles;
}
AreaUnits_AreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_AreaUnits_AreaUnitsEnum_SquareKilometers() {
  return AreaUnits::SquareKilometers;
}

// LengthUnits_LengthUnitsEnum
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Feet() {
  return LengthUnits::Feet;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Inches() {
  return LengthUnits::Inches;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Centimeters() {
  return LengthUnits::Centimeters;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Meters() {
  return LengthUnits::Meters;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Chains() {
  return LengthUnits::Chains;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Miles() {
  return LengthUnits::Miles;
}
LengthUnits_LengthUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LengthUnits_LengthUnitsEnum_Kilometers() {
  return LengthUnits::Kilometers;
}

// LoadingUnits_LoadingUnitsEnum
LoadingUnits_LoadingUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LoadingUnits_LoadingUnitsEnum_PoundsPerSquareFoot() {
  return LoadingUnits::PoundsPerSquareFoot;
}
LoadingUnits_LoadingUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LoadingUnits_LoadingUnitsEnum_TonsPerAcre() {
  return LoadingUnits::TonsPerAcre;
}
LoadingUnits_LoadingUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LoadingUnits_LoadingUnitsEnum_TonnesPerHectare() {
  return LoadingUnits::TonnesPerHectare;
}
LoadingUnits_LoadingUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_LoadingUnits_LoadingUnitsEnum_KilogramsPerSquareMeter() {
  return LoadingUnits::KilogramsPerSquareMeter;
}

// SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum
SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareFeetOverCubicFeet() {
  return SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet;
}
SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareMetersOverCubicMeters() {
  return SurfaceAreaToVolumeUnits::SquareMetersOverCubicMeters;
}
SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareInchesOverCubicInches() {
  return SurfaceAreaToVolumeUnits::SquareInchesOverCubicInches;
}
SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SurfaceAreaToVolumeUnits_SurfaceAreaToVolumeUnitsEnum_SquareCentimetersOverCubicCentimers() {
  return SurfaceAreaToVolumeUnits::SquareCentimetersOverCubicCentimers;
}

// CoverUnits_CoverUnitsEnum
CoverUnits_CoverUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_CoverUnits_CoverUnitsEnum_Fraction() {
  return CoverUnits::Fraction;
}
CoverUnits_CoverUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_CoverUnits_CoverUnitsEnum_Percent() {
  return CoverUnits::Percent;
}

// SpeedUnits_SpeedUnitsEnum
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_FeetPerMinute() {
  return SpeedUnits::FeetPerMinute;
}
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_ChainsPerHour() {
  return SpeedUnits::ChainsPerHour;
}
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_MetersPerSecond() {
  return SpeedUnits::MetersPerSecond;
}
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_MetersPerMinute() {
  return SpeedUnits::MetersPerMinute;
}
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_MilesPerHour() {
  return SpeedUnits::MilesPerHour;
}
SpeedUnits_SpeedUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SpeedUnits_SpeedUnitsEnum_KilometersPerHour() {
  return SpeedUnits::KilometersPerHour;
}

// ProbabilityUnits_ProbabilityUnitsEnum
ProbabilityUnits_ProbabilityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_ProbabilityUnits_ProbabilityUnitsEnum_Fraction() {
  return ProbabilityUnits::Fraction;
}
ProbabilityUnits_ProbabilityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_ProbabilityUnits_ProbabilityUnitsEnum_Percent() {
  return ProbabilityUnits::Percent;
}

// MoistureUnits_MoistureUnitsEnum
MoistureUnits_MoistureUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_MoistureUnits_MoistureUnitsEnum_Fraction() {
  return MoistureUnits::Fraction;
}
MoistureUnits_MoistureUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_MoistureUnits_MoistureUnitsEnum_Percent() {
  return MoistureUnits::Percent;
}

// SlopeUnits_SlopeUnitsEnum
SlopeUnits_SlopeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SlopeUnits_SlopeUnitsEnum_Degrees() {
  return SlopeUnits::Degrees;
}
SlopeUnits_SlopeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_SlopeUnits_SlopeUnitsEnum_Percent() {
  return SlopeUnits::Percent;
}

// DensityUnits_DensityUnitsEnum
DensityUnits_DensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_DensityUnits_DensityUnitsEnum_PoundsPerCubicFoot() {
  return DensityUnits::PoundsPerCubicFoot;
}
DensityUnits_DensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_DensityUnits_DensityUnitsEnum_KilogramsPerCubicMeter() {
  return DensityUnits::KilogramsPerCubicMeter;
}

// HeatOfCombustionUnits_HeatOfCombustionUnitsEnum
HeatOfCombustionUnits_HeatOfCombustionUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatOfCombustionUnits_HeatOfCombustionUnitsEnum_BtusPerPound() {
  return HeatOfCombustionUnits::BtusPerPound;
}
HeatOfCombustionUnits_HeatOfCombustionUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatOfCombustionUnits_HeatOfCombustionUnitsEnum_KilojoulesPerKilogram() {
  return HeatOfCombustionUnits::KilojoulesPerKilogram;
}

// HeatSinkUnits_HeatSinkUnitsEnum
HeatSinkUnits_HeatSinkUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSinkUnits_HeatSinkUnitsEnum_BtusPerCubicFoot() {
  return HeatSinkUnits::BtusPerCubicFoot;
}
HeatSinkUnits_HeatSinkUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSinkUnits_HeatSinkUnitsEnum_KilojoulesPerCubicMeter() {
  return HeatSinkUnits::KilojoulesPerCubicMeter;
}

// HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum
HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_BtusPerSquareFoot() {
  return HeatPerUnitAreaUnits::BtusPerSquareFoot;
}
HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_KilojoulesPerSquareMeterPerSecond() {
  return HeatPerUnitAreaUnits::KilojoulesPerSquareMeterPerSecond;
}
HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatPerUnitAreaUnits_HeatPerUnitAreaUnitsEnum_KilowattsPerSquareMeter() {
  return HeatPerUnitAreaUnits::KilowattsPerSquareMeter;
}

// HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum
HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_BtusPerSquareFootPerMinute() {
  return HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute;
}
HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_BtusPerSquareFootPerSecond() {
  return HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerSecond;
}
HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilojoulesPerSquareMeterPerSecond() {
  return HeatSourceAndReactionIntensityUnits::KilojoulesPerSquareMeterPerSecond;
}
HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilojoulesPerSquareMeterPerMinute() {
  return HeatSourceAndReactionIntensityUnits::KilojoulesPerSquareMeterPerMinute;
}
HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_HeatSourceAndReactionIntensityUnits_HeatSourceAndReactionIntensityUnitsEnum_KilowattsPerSquareMeter() {
  return HeatSourceAndReactionIntensityUnits::KilowattsPerSquareMeter;
}

// FirelineIntensityUnits_FirelineIntensityUnitsEnum
FirelineIntensityUnits_FirelineIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_BtusPerFootPerSecond() {
  return FirelineIntensityUnits::BtusPerFootPerSecond;
}
FirelineIntensityUnits_FirelineIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_BtusPerFootPerMinute() {
  return FirelineIntensityUnits::BtusPerFootPerMinute;
}
FirelineIntensityUnits_FirelineIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilojoulesPerMeterPerSecond() {
  return FirelineIntensityUnits::KilojoulesPerMeterPerSecond;
}
FirelineIntensityUnits_FirelineIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilojoulesPerMeterPerMinute() {
  return FirelineIntensityUnits::KilojoulesPerMeterPerMinute;
}
FirelineIntensityUnits_FirelineIntensityUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_FirelineIntensityUnits_FirelineIntensityUnitsEnum_KilowattsPerMeter() {
  return FirelineIntensityUnits::KilowattsPerMeter;
}

// TemperatureUnits_TemperatureUnitsEnum
TemperatureUnits_TemperatureUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Fahrenheit() {
  return TemperatureUnits::Fahrenheit;
}
TemperatureUnits_TemperatureUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Celsius() {
  return TemperatureUnits::Celsius;
}
TemperatureUnits_TemperatureUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TemperatureUnits_TemperatureUnitsEnum_Kelvin() {
  return TemperatureUnits::Kelvin;
}

// TimeUnits_TimeUnitsEnum
TimeUnits_TimeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TimeUnits_TimeUnitsEnum_Minutes() {
  return TimeUnits::Minutes;
}
TimeUnits_TimeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TimeUnits_TimeUnitsEnum_Seconds() {
  return TimeUnits::Seconds;
}
TimeUnits_TimeUnitsEnum EMSCRIPTEN_KEEPALIVE emscripten_enum_TimeUnits_TimeUnitsEnum_Hours() {
  return TimeUnits::Hours;
}

// ContainTactic
ContainTactic EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainTactic_HeadAttack() {
  return HeadAttack;
}
ContainTactic EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainTactic_RearAttack() {
  return RearAttack;
}

// ContainStatus
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Unreported() {
  return Unreported;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Reported() {
  return Reported;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Attacked() {
  return Attacked;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Contained() {
  return Contained;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Overrun() {
  return Overrun;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Exhausted() {
  return Exhausted;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_Overflow() {
  return Overflow;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_SizeLimitExceeded() {
  return SizeLimitExceeded;
}
ContainStatus EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainStatus_TimeLimitExceeded() {
  return TimeLimitExceeded;
}

// ContainFlank
ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainFlank_LeftFlank() {
  return LeftFlank;
}
ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainFlank_RightFlank() {
  return RightFlank;
}
ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainFlank_BothFlanks() {
  return BothFlanks;
}
ContainFlank EMSCRIPTEN_KEEPALIVE emscripten_enum_ContainFlank_NeitherFlank() {
  return NeitherFlank;
}

// SpotTreeSpecies
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_ENGELMANN_SPRUCE() {
  return ENGELMANN_SPRUCE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_DOUGLAS_FIR() {
  return DOUGLAS_FIR;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_SUBALPINE_FIR() {
  return SUBALPINE_FIR;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_WESTERN_HEMLOCK() {
  return WESTERN_HEMLOCK;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_PONDEROSA_PINE() {
  return PONDEROSA_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_LODGEPOLE_PINE() {
  return LODGEPOLE_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_WESTERN_WHITE_PINE() {
  return WESTERN_WHITE_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_GRAND_FIR() {
  return GRAND_FIR;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_BALSAM_FIR() {
  return BALSAM_FIR;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_SLASH_PINE() {
  return SLASH_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_LONGLEAF_PINE() {
  return LONGLEAF_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_POND_PINE() {
  return POND_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_SHORTLEAF_PINE() {
  return SHORTLEAF_PINE;
}
SpotTreeSpecies EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotTreeSpecies_LOBLOLLY_PINE() {
  return LOBLOLLY_PINE;
}

// SpotFireLocation
SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotFireLocation_MIDSLOPE_WINDWARD() {
  return MIDSLOPE_WINDWARD;
}
SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotFireLocation_VALLEY_BOTTOM() {
  return VALLEY_BOTTOM;
}
SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotFireLocation_MIDSLOPE_LEEWARD() {
  return MIDSLOPE_LEEWARD;
}
SpotFireLocation EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotFireLocation_RIDGE_TOP() {
  return RIDGE_TOP;
}

// SpotArrayConstants
SpotArrayConstants EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotArrayConstants_NUM_COLS() {
  return NUM_COLS;
}
SpotArrayConstants EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotArrayConstants_NUM_FIREBRAND_ROWS() {
  return NUM_FIREBRAND_ROWS;
}
SpotArrayConstants EMSCRIPTEN_KEEPALIVE emscripten_enum_SpotArrayConstants_NUM_SPECIES() {
  return NUM_SPECIES;
}

}

