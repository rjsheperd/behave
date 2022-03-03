//------------------------------------------------------------------------------
/*! \file worksheet.h
    \author Copyright (C) 2022 Spatial Informatics Group
    \license This is released under the GNU Public License 2.
 */

#ifndef _WORKSHEET_H_INCLUDED_
#define _WORKSHEET_H_INCLUDED_

#include <stdlib.h>
#include <stdlib.h>
#include <string>
#include <unordered_map>

class Worksheet
{
public:
  Worksheet( void );

  // Continuous Variables
  void addContinuousVar(const char * name);
  void addContinuousVar(const char * name, double value);

  // Discrete Variables
  void addDiscreteVar(const char * name);
  void addDiscreteVar(const char * name, const char * value);

  // Text Variables
  void addTextVar(const char * name);
  void addTextVar(const char * name, const char * value);

  // Accessors
  void getTextVar(const char * name, char * result);
  void getDiscreteVar(const char * name, char * result);
  double getContinuousVar(const char * name);

private:
  std::unordered_map<std::string, double> continous_variables;
  std::unordered_map<std::string, std::string> discrete_variables;
  std::unordered_map<std::string, std::string> text_variables;
};

#endif
//------------------------------------------------------------------------------
//  End of worksheet.h
//------------------------------------------------------------------------------
