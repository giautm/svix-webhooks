/*
 * Svix API
 *
 * Welcome to the Svix API documentation!  Useful links: [Homepage](https://www.svix.com) | [Support email](mailto:support+docs@svix.com) | [Blog](https://www.svix.com/blog/) | [Slack Community](https://www.svix.com/slack/)  # Introduction  This is the reference documentation and schemas for the [Svix webhook service](https://www.svix.com) API. For tutorials and other documentation please refer to [the documentation](https://docs.svix.com).  ## Main concepts  In Svix you have four important entities you will be interacting with:  - `messages`: these are the webhooks being sent. They can have contents and a few other properties. - `application`: this is where `messages` are sent to. Usually you want to create one application for each user on your platform. - `endpoint`: endpoints are the URLs messages will be sent to. Each application can have multiple `endpoints` and each message sent to that application will be sent to all of them (unless they are not subscribed to the sent event type). - `event-type`: event types are identifiers denoting the type of the message being sent. Event types are primarily used to decide which events are sent to which endpoint.   ## Authentication  Get your authentication token (`AUTH_TOKEN`) from the [Svix dashboard](https://dashboard.svix.com) and use it as part of the `Authorization` header as such: `Authorization: Bearer ${AUTH_TOKEN}`. For more information on authentication, please refer to the [authentication token docs](https://docs.svix.com/api-keys).  <SecurityDefinitions />   ## Code samples  The code samples assume you already have the respective libraries installed and you know how to use them. For the latest information on how to do that, please refer to [the documentation](https://docs.svix.com/).   ## Idempotency  Svix supports [idempotency](https://en.wikipedia.org/wiki/Idempotence) for safely retrying requests without accidentally performing the same operation twice. This is useful when an API call is disrupted in transit and you do not receive a response.  To perform an idempotent request, pass the idempotency key in the `Idempotency-Key` header to the request. The idempotency key should be a unique value generated by the client. You can create the key in however way you like, though we suggest using UUID v4, or any other string with enough entropy to avoid collisions.  Svix's idempotency works by saving the resulting status code and body of the first request made for any given idempotency key for any successful request. Subsequent requests with the same key return the same result.  Please note that idempotency is only supported for `POST` requests.   ## Cross-Origin Resource Sharing  This API features Cross-Origin Resource Sharing (CORS) implemented in compliance with [W3C spec](https://www.w3.org/TR/cors/). And that allows cross-domain communication from the browser. All responses have a wildcard same-origin which makes them completely public and accessible to everyone, including any code on any site. 
 *
 * API version: 1.7.0
 */

// Code generated by OpenAPI Generator (https://openapi-generator.tech); DO NOT EDIT.

package openapi

import (
	"encoding/json"
)

// TemplateIn struct for TemplateIn
type TemplateIn struct {
	Description *string `json:"description,omitempty"`
	FilterTypes []string `json:"filterTypes,omitempty"`
	Instructions *string `json:"instructions,omitempty"`
	InstructionsLink NullableString `json:"instructionsLink,omitempty"`
	Logo string `json:"logo"`
	Name string `json:"name"`
	Transformation string `json:"transformation"`
}

// NewTemplateIn instantiates a new TemplateIn object
// This constructor will assign default values to properties that have it defined,
// and makes sure properties required by API are set, but the set of arguments
// will change when the set of required properties is changed
func NewTemplateIn(logo string, name string, transformation string) *TemplateIn {
	this := TemplateIn{}
	var description string = ""
	this.Description = &description
	var instructions string = ""
	this.Instructions = &instructions
	var instructionsLink string = "null"
	this.InstructionsLink = *NewNullableString(&instructionsLink)
	this.Logo = logo
	this.Name = name
	this.Transformation = transformation
	return &this
}

// NewTemplateInWithDefaults instantiates a new TemplateIn object
// This constructor will only assign default values to properties that have it defined,
// but it doesn't guarantee that properties required by API are set
func NewTemplateInWithDefaults() *TemplateIn {
	this := TemplateIn{}
	var description string = ""
	this.Description = &description
	var instructions string = ""
	this.Instructions = &instructions
	var instructionsLink string = "null"
	this.InstructionsLink = *NewNullableString(&instructionsLink)
	return &this
}

// GetDescription returns the Description field value if set, zero value otherwise.
func (o *TemplateIn) GetDescription() string {
	if o == nil || o.Description == nil {
		var ret string
		return ret
	}
	return *o.Description
}

// GetDescriptionOk returns a tuple with the Description field value if set, nil otherwise
// and a boolean to check if the value has been set.
func (o *TemplateIn) GetDescriptionOk() (*string, bool) {
	if o == nil || o.Description == nil {
		return nil, false
	}
	return o.Description, true
}

// HasDescription returns a boolean if a field has been set.
func (o *TemplateIn) HasDescription() bool {
	if o != nil && o.Description != nil {
		return true
	}

	return false
}

// SetDescription gets a reference to the given string and assigns it to the Description field.
func (o *TemplateIn) SetDescription(v string) {
	o.Description = &v
}

// GetFilterTypes returns the FilterTypes field value if set, zero value otherwise (both if not set or set to explicit null).
func (o *TemplateIn) GetFilterTypes() []string {
	if o == nil  {
		var ret []string
		return ret
	}
	return o.FilterTypes
}

// GetFilterTypesOk returns a tuple with the FilterTypes field value if set, nil otherwise
// and a boolean to check if the value has been set.
// NOTE: If the value is an explicit nil, `nil, true` will be returned
func (o *TemplateIn) GetFilterTypesOk() (*[]string, bool) {
	if o == nil || o.FilterTypes == nil {
		return nil, false
	}
	return &o.FilterTypes, true
}

// HasFilterTypes returns a boolean if a field has been set.
func (o *TemplateIn) HasFilterTypes() bool {
	if o != nil && o.FilterTypes != nil {
		return true
	}

	return false
}

// SetFilterTypes gets a reference to the given []string and assigns it to the FilterTypes field.
func (o *TemplateIn) SetFilterTypes(v []string) {
	o.FilterTypes = v
}

// GetInstructions returns the Instructions field value if set, zero value otherwise.
func (o *TemplateIn) GetInstructions() string {
	if o == nil || o.Instructions == nil {
		var ret string
		return ret
	}
	return *o.Instructions
}

// GetInstructionsOk returns a tuple with the Instructions field value if set, nil otherwise
// and a boolean to check if the value has been set.
func (o *TemplateIn) GetInstructionsOk() (*string, bool) {
	if o == nil || o.Instructions == nil {
		return nil, false
	}
	return o.Instructions, true
}

// HasInstructions returns a boolean if a field has been set.
func (o *TemplateIn) HasInstructions() bool {
	if o != nil && o.Instructions != nil {
		return true
	}

	return false
}

// SetInstructions gets a reference to the given string and assigns it to the Instructions field.
func (o *TemplateIn) SetInstructions(v string) {
	o.Instructions = &v
}

// GetInstructionsLink returns the InstructionsLink field value if set, zero value otherwise (both if not set or set to explicit null).
func (o *TemplateIn) GetInstructionsLink() string {
	if o == nil || o.InstructionsLink.Get() == nil {
		var ret string
		return ret
	}
	return *o.InstructionsLink.Get()
}

// GetInstructionsLinkOk returns a tuple with the InstructionsLink field value if set, nil otherwise
// and a boolean to check if the value has been set.
// NOTE: If the value is an explicit nil, `nil, true` will be returned
func (o *TemplateIn) GetInstructionsLinkOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return o.InstructionsLink.Get(), o.InstructionsLink.IsSet()
}

// HasInstructionsLink returns a boolean if a field has been set.
func (o *TemplateIn) HasInstructionsLink() bool {
	if o != nil && o.InstructionsLink.IsSet() {
		return true
	}

	return false
}

// SetInstructionsLink gets a reference to the given NullableString and assigns it to the InstructionsLink field.
func (o *TemplateIn) SetInstructionsLink(v string) {
	o.InstructionsLink.Set(&v)
}
// SetInstructionsLinkNil sets the value for InstructionsLink to be an explicit nil
func (o *TemplateIn) SetInstructionsLinkNil() {
	o.InstructionsLink.Set(nil)
}

// UnsetInstructionsLink ensures that no value is present for InstructionsLink, not even an explicit nil
func (o *TemplateIn) UnsetInstructionsLink() {
	o.InstructionsLink.Unset()
}

// GetLogo returns the Logo field value
func (o *TemplateIn) GetLogo() string {
	if o == nil {
		var ret string
		return ret
	}

	return o.Logo
}

// GetLogoOk returns a tuple with the Logo field value
// and a boolean to check if the value has been set.
func (o *TemplateIn) GetLogoOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Logo, true
}

// SetLogo sets field value
func (o *TemplateIn) SetLogo(v string) {
	o.Logo = v
}

// GetName returns the Name field value
func (o *TemplateIn) GetName() string {
	if o == nil {
		var ret string
		return ret
	}

	return o.Name
}

// GetNameOk returns a tuple with the Name field value
// and a boolean to check if the value has been set.
func (o *TemplateIn) GetNameOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Name, true
}

// SetName sets field value
func (o *TemplateIn) SetName(v string) {
	o.Name = v
}

// GetTransformation returns the Transformation field value
func (o *TemplateIn) GetTransformation() string {
	if o == nil {
		var ret string
		return ret
	}

	return o.Transformation
}

// GetTransformationOk returns a tuple with the Transformation field value
// and a boolean to check if the value has been set.
func (o *TemplateIn) GetTransformationOk() (*string, bool) {
	if o == nil  {
		return nil, false
	}
	return &o.Transformation, true
}

// SetTransformation sets field value
func (o *TemplateIn) SetTransformation(v string) {
	o.Transformation = v
}

func (o TemplateIn) MarshalJSON() ([]byte, error) {
	toSerialize := map[string]interface{}{}
	if o.Description != nil {
		toSerialize["description"] = o.Description
	}
	if o.FilterTypes != nil {
		toSerialize["filterTypes"] = o.FilterTypes
	}
	if o.Instructions != nil {
		toSerialize["instructions"] = o.Instructions
	}
	if o.InstructionsLink.IsSet() {
		toSerialize["instructionsLink"] = o.InstructionsLink.Get()
	}
	if true {
		toSerialize["logo"] = o.Logo
	}
	if true {
		toSerialize["name"] = o.Name
	}
	if true {
		toSerialize["transformation"] = o.Transformation
	}
	return json.Marshal(toSerialize)
}

type NullableTemplateIn struct {
	value *TemplateIn
	isSet bool
}

func (v NullableTemplateIn) Get() *TemplateIn {
	return v.value
}

func (v *NullableTemplateIn) Set(val *TemplateIn) {
	v.value = val
	v.isSet = true
}

func (v NullableTemplateIn) IsSet() bool {
	return v.isSet
}

func (v *NullableTemplateIn) Unset() {
	v.value = nil
	v.isSet = false
}

func NewNullableTemplateIn(val *TemplateIn) *NullableTemplateIn {
	return &NullableTemplateIn{value: val, isSet: true}
}

func (v NullableTemplateIn) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.value)
}

func (v *NullableTemplateIn) UnmarshalJSON(src []byte) error {
	v.isSet = true
	return json.Unmarshal(src, &v.value)
}


