//------------------------------------------------------------------------------
/*! \file worksheet.cpp
    \author Copyright (C) 2022 Spatial Informatics Group
    \license This is released under the GNU Public License 2.
 */

// Custom include files
#include "worksheet.h"

// C include files
#include <string.h>  // strcopy

// Standard include files
#include <memory>    // std::shared_ptr
#include <iostream>  // std::cerr
#include <stdexcept> // std::out_of_range

Worksheet::Worksheet(void) {}

// Discrete Variables
void Worksheet::addDiscreteVar(const char * name) {
  discrete_variables.insert({name, ""});
}

void Worksheet::addDiscreteVar(const char * name, const char * value) {
  discrete_variables.insert({name, value});
}

// Continuous Variables
void Worksheet::addContinuousVar(const char * name) {
  continous_variables.insert({name, 0.0});
}

void Worksheet::addContinuousVar(const char * name, double value) {
  continous_variables.insert({name, value});
}

// Text Variables
void Worksheet::addTextVar(const char * name) {
  text_variables.insert({name, ""});
}

void Worksheet::addTextVar(const char * name, const char * value) {
  text_variables.insert({name, value});
}

// Accessors
void Worksheet::getTextVar(const char * name, char * result) {
  std::string s_result = "";
  try {
    s_result = text_variables.at(name);
  }
  catch (const std::out_of_range& oor) {
    std::cerr << "Out of Range error: " << oor.what() << '\n';
  }
  strcpy(result, s_result.data());
}

void Worksheet::getDiscreteVar(const char * name, char * result) {
  std::string s_result = "";
  try {
    s_result = discrete_variables.at(name);
  }
  catch (const std::out_of_range& oor) {
    std::cerr << "Out of Range error: " << oor.what() << '\n';
  }
  strcpy(result, s_result.data());
}

double Worksheet::getContinuousVar(const char * name) {
  double result = 0.0;
  try {
    result = continous_variables.at(name);
  }
  catch (const std::out_of_range& oor) {
    std::cerr << "Out of Range error: " << oor.what() << '\n';
  }
  return result;
}
