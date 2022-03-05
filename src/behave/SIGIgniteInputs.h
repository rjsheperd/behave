/******************************************************************************
*
* Project:  CodeBlocks
* Purpose:  Class for handling the inputs required for Behave's Ignite module
* Author:   William Chatham <wchatham@fs.fed.us>
* Author:   Richard Sheperd <rsheperd@sig-gis.com>
*
*******************************************************************************
*
* THIS SOFTWARE WAS DEVELOPED AT THE ROCKY MOUNTAIN RESEARCH STATION (RMRS)
* MISSOULA FIRE SCIENCES LABORATORY BY EMPLOYEES OF THE FEDERAL GOVERNMENT
* IN THE COURSE OF THEIR OFFICIAL DUTIES. PURSUANT TO TITLE 17 SECTION 105
* OF THE UNITED STATES CODE, THIS SOFTWARE IS NOT SUBJECT TO COPYRIGHT
* PROTECTION AND IS IN THE PUBLIC DOMAIN. RMRS MISSOULA FIRE SCIENCES
* LABORATORY ASSUMES NO RESPONSIBILITY WHATSOEVER FOR ITS USE BY OTHER
* PARTIES,  AND MAKES NO GUARANTEES, EXPRESSED OR IMPLIED, ABOUT ITS QUALITY,
* RELIABILITY, OR ANY OTHER CHARACTERISTIC.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
* OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
* THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
* FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
* DEALINGS IN THE SOFTWARE.
*
******************************************************************************/

#ifndef _SIGIGNITEINPUTS_H_
#define _SIGIGNITEINPUTS_H_

#include "behaveUnits.h"

enum IgnitionFuelBedType
{
    PonderosaPineLitter = 0, //Ponderosa Pine Litter
    PunkyWoodRottenChunky = 1, // Punky wood, rotten, chunky
    PunkyWoodPowderDeep = 2, // Punky wood powder, deep (4.8 cm)
    PunkWoodPowderShallow = 3, // Punk wood powder, shallow (2.4 cm)
    LodgepolePineDuff = 4, // Lodgepole pine duff
    DouglasFirDuff = 5, //  Douglas - fir duff
    HighAltitudeMixed = 6, // High altitude mixed (mainly Engelmann spruce)
    PeatMoss = 7  // Peat moss (commercial)
};

enum LightningCharge
{
    Negative = 0,
    Positive = 1,
    Unknown = 2
};

class SIGIgniteInputs
{
public:
    SIGIgniteInputs();
    ~SIGIgniteInputs();

    void initializeMembers();

    void updateIgniteInputs(double moistureOneHour, double moistureHundredHour, MoistureUnits::MoistureUnitsEnum moistureUnits,
        double airTemperature, TemperatureUnits::TemperatureUnitsEnum temperatureUnits, double sunShade, CoverUnits::CoverUnitsEnum sunShadeUnits,
        IgnitionFuelBedType fuelBedType, double duffDepth, LengthUnits::LengthUnitsEnum duffDepthUnits,
        LightningCharge lightningChargeType);

    void setMoistureHundredHour(double hundredHourMoisture, MoistureUnits::MoistureUnitsEnum moistureUnits);
    void setMoistureOneHour(double moistureOneHour, MoistureUnits::MoistureUnitsEnum moistureUnits);
    void setAirTemperature(double airTemperature, TemperatureUnits::TemperatureUnitsEnum temperatureUnits);
    void setSunShade(double sunShade, CoverUnits::CoverUnitsEnum sunShadeUnits);
    void setIgnitionFuelBedType(IgnitionFuelBedType fuelBedType_);
    void setDuffDepth(double duffDepth, LengthUnits::LengthUnitsEnum lengthUnits);

    void setLightningChargeType(LightningCharge lightningChargeType);

    double getAirTemperature(TemperatureUnits::TemperatureUnitsEnum desiredUnits);
    double getMoistureOneHour(MoistureUnits::MoistureUnitsEnum desiredUnits);
    double getMoistureHundredHour(MoistureUnits::MoistureUnitsEnum desiredUnits);
    double getSunShade(CoverUnits::CoverUnitsEnum desiredUnits);
    IgnitionFuelBedType getIgnitionFuelBedType();
    double getDuffDepth(LengthUnits::LengthUnitsEnum desiredUnits);
    LightningCharge getLightningChargeType();

private:
    double moistureOneHour_;
    double moistureHundredHour_;
    double airTemperature_;
    double sunShade_;
    IgnitionFuelBedType fuelBedType_;
    double duffDepth_;

    LightningCharge lightningChargeType_;
};

#endif // IGNITEINPUTS_H

