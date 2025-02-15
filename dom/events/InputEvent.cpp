/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=8 sts=2 et sw=2 tw=80: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "mozilla/dom/InputEvent.h"
#include "mozilla/TextEvents.h"
#include "prtime.h"

namespace mozilla {
namespace dom {

InputEvent::InputEvent(EventTarget* aOwner, nsPresContext* aPresContext,
                       InternalEditorInputEvent* aEvent)
    : UIEvent(aOwner, aPresContext,
              aEvent
                  ? aEvent
                  : new InternalEditorInputEvent(false, eVoidEvent, nullptr)) {
  NS_ASSERTION(mEvent->mClass == eEditorInputEventClass, "event type mismatch");

  if (aEvent) {
    mEventIsInternal = false;
  } else {
    mEventIsInternal = true;
    mEvent->mTime = PR_Now();
  }
}

void InputEvent::GetInputType(nsAString& aInputType) {
  InternalEditorInputEvent* editorInputEvent = mEvent->AsEditorInputEvent();
  MOZ_ASSERT(editorInputEvent);
  if (editorInputEvent->mInputType == EditorInputType::eUnknown) {
    aInputType = mInputTypeValue;
  } else {
    editorInputEvent->GetDOMInputTypeName(aInputType);
  }
}

bool InputEvent::IsComposing() {
  return mEvent->AsEditorInputEvent()->mIsComposing;
}

already_AddRefed<InputEvent> InputEvent::Constructor(
    const GlobalObject& aGlobal, const nsAString& aType,
    const InputEventInit& aParam, ErrorResult& aRv) {
  nsCOMPtr<EventTarget> t = do_QueryInterface(aGlobal.GetAsSupports());
  RefPtr<InputEvent> e = new InputEvent(t, nullptr, nullptr);
  bool trusted = e->Init(t);
  e->InitUIEvent(aType, aParam.mBubbles, aParam.mCancelable, aParam.mView,
                 aParam.mDetail);
  InternalEditorInputEvent* internalEvent = e->mEvent->AsEditorInputEvent();
  internalEvent->mInputType =
      InternalEditorInputEvent::GetEditorInputType(aParam.mInputType);
  if (internalEvent->mInputType == EditorInputType::eUnknown) {
    e->mInputTypeValue = aParam.mInputType;
  }
  internalEvent->mIsComposing = aParam.mIsComposing;
  e->SetTrusted(trusted);
  e->SetComposed(aParam.mComposed);
  return e.forget();
}

}  // namespace dom
}  // namespace mozilla

using namespace mozilla;
using namespace mozilla::dom;

already_AddRefed<InputEvent> NS_NewDOMInputEvent(
    EventTarget* aOwner, nsPresContext* aPresContext,
    InternalEditorInputEvent* aEvent) {
  RefPtr<InputEvent> it = new InputEvent(aOwner, aPresContext, aEvent);
  return it.forget();
}
