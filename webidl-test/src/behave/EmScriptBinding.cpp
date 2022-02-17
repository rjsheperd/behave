#include <iomanip>
#include <iostream>
#include <math.h>

#include "behaveRun.h"
#include "fuelModelSet.h"
#include "EmScriptBinding.h"

double EmScriptBinding::getSpreadRateEM()
{
    // Surface Fire Inputs;
    int fuelModelNumber = 0;
    double moistureOneHour = 0.0;
    double moistureTenHour = 0.0;
    double moistureHundredHour = 0.0;
    double moistureLiveHerbaceous = 0.0;
    double moistureLiveWoody = 0.0;
    double windSpeed = 0.0;
    SpeedUnits::SpeedUnitsEnum windSpeedUnits = SpeedUnits::MilesPerHour;
    double windDirection = 0;
    WindHeightInputMode::WindHeightInputModeEnum windHeightInputMode = WindHeightInputMode::DirectMidflame;
    double slope = 0.0;
    double aspect = 0.0;
    double directionOfMaxSpread = 0;
    double flameLength = 0;
    double directionOfInterest = 0;
    double spreadRate = 0;

    // Wind adjustment factor parameters
    double canopyCover = 0.0;
    double canopyHeight = 0.0;
    double crownRatio = 0.0;

    FuelModelSet fuelModelSet;
    BehaveRun behave(fuelModelSet);

    // Setting the wind and spread angle input mode (default is upslope)
    behave.surface.setWindAndSpreadOrientationMode(WindAndSpreadOrientationMode::RelativeToNorth);

    // Checking  wind and spread angle input mode
    WindAndSpreadOrientationMode::WindAndSpreadOrientationModeEnum windAndSpreadOrientationMode = behave.surface.getWindAndSpreadOrientationMode();
    std::cout << "Wind and spread direction are in degrees clockwise relative to ";
    if (windAndSpreadOrientationMode == WindAndSpreadOrientationMode::RelativeToUpslope)
    {
        std::cout << "upslope" << std::endl << std::endl;
    }
    if (windAndSpreadOrientationMode == WindAndSpreadOrientationMode::RelativeToNorth)
    {
        std::cout << "compass north" << std::endl << std::endl;
    }

    int i = 0;
    fuelModelNumber = i;
    moistureOneHour = 6;
    moistureTenHour = 7;
    moistureHundredHour = 8;
    moistureLiveHerbaceous = 60;
    moistureLiveWoody = 90;
    TwoFuelModelsMethod::TwoFuelModelsMethodEnum  twoFuelModelsMethod = TwoFuelModelsMethod::TwoFimensional;
    windSpeed = 5;
    windHeightInputMode = WindHeightInputMode::DirectMidflame;
    windDirection = 180;
    slope = 30;
    SlopeUnits::SlopeUnitsEnum slopeUnits = SlopeUnits::Percent;
    aspect = 250;
    directionOfInterest = 0;


    // Two Fuel Models test
    int firstFuelModelNumber = 0;
    int secondFuelModelNumber = 0;
    double firstFuelModelCoverage = 0.0;
    firstFuelModelNumber = 1;
    secondFuelModelNumber = 124;
    firstFuelModelCoverage = 0 + (.10 * i);
    CoverUnits::CoverUnitsEnum firstFuelModelCoverageUnits = CoverUnits::Percent;
    CoverUnits::CoverUnitsEnum canopyCoverUnits = CoverUnits::Percent;
    LengthUnits::LengthUnitsEnum canopyHeightUnits = LengthUnits::Feet;

    behave.surface.updateSurfaceInputsForTwoFuelModels(firstFuelModelNumber, secondFuelModelNumber, moistureOneHour, moistureTenHour,
        moistureHundredHour, moistureLiveHerbaceous, moistureLiveWoody, MoistureUnits::Percent, windSpeed, windSpeedUnits, windHeightInputMode,
        windDirection, windAndSpreadOrientationMode, firstFuelModelCoverage, firstFuelModelCoverageUnits, twoFuelModelsMethod, slope,
        slopeUnits, aspect, canopyCover, canopyCoverUnits, canopyHeight, canopyHeightUnits, crownRatio);
    behave.surface.doSurfaceRunInDirectionOfInterest(directionOfInterest);
    spreadRate = behave.surface.getSpreadRate(SpeedUnits::ChainsPerHour);
    //spreadRate = floor(spreadRate * 10 + 0.5) / 10;
    std::cout << "Spread rate for the two fuel models " << firstFuelModelNumber << " and " << secondFuelModelNumber << " with first fuel coverage " << firstFuelModelCoverage * 100 << "%" << std::endl;
    std::cout << "is " << spreadRate << " ch/hr" << std::endl;
    flameLength = behave.surface.getFlameLength(LengthUnits::Feet);
    //flameLength = floor(flameLength * 10 + 0.5) / 10;
    std::cout << "Flame length for the two fuel models " << firstFuelModelNumber << " and " << secondFuelModelNumber << " is " << flameLength << " ft" << std::endl;

    // Direction of Max Spread test
    directionOfMaxSpread = behave.surface.getDirectionOfMaxSpread();
    std::cout << "Direction of maximum spread is " << directionOfMaxSpread << " degrees" << std::endl << std::endl;
    

    return spreadRate;
}

double getSpreadRate()
{
    return 0.0;
}
