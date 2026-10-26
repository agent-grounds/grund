---
title: "Widget Synchronization over HTTP"
abbrev: "Widget Sync"
category: std

docname: draft-ietf-wsync-widget-sync-latest
submissiontype: IETF
number:
date:
consensus: true
v: 3
area: "Applications and Real-Time"
workgroup: "Widget Synchronization"
keyword:
 - synchronization
 - entity tags
venue:
  group: "Widget Synchronization"
  type: "Working Group"
  mail: "wsync@lists.example"
  github: "wsync-example/widget-sync"

author:
 -
    fullname: "Mira Castellanos"
    organization: "Example Widget Works"
    email: "mira@widgets.example"
 -
    fullname: "Tomas Lindqvist"
    organization: "Example Cloud"
    email: "tomas@cloud.example"

normative:
  RFC2119:
  RFC6585:
  RFC8174:
  RFC9110:
  RFC9457:

informative:

...

# draft-ietf-wsync-widget-sync: Widget Synchronization over HTTP

--- abstract

This document defines Widget Sync, a protocol that keeps a client's copy
of a collection of widgets consistent with the copy an HTTP server holds.
A client lists only what changed since the last listing it saw, and
updates a widget only if the server's widget is still the one the client
last received, so that a concurrent edit is reported instead of lost.

--- middle

## 1. Introduction

Applications that edit one collection from several devices need each
device to learn what changed elsewhere, and to avoid overwriting an edit
it has not seen. Listing the whole collection answers the first need at a
cost that grows with the collection; unconditional updates fail the second.

This document defines a widget resource and its entity tag
(§draft-ietf-wsync-widget-sync.3), a change listing a client can resume
(§draft-ietf-wsync-widget-sync.4.1), and conditional updates that turn a
lost update into a reported conflict (§draft-ietf-wsync-widget-sync.4.2,
§draft-ietf-wsync-widget-sync.4.3).

## 2. Conventions and Definitions

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "NOT RECOMMENDED", "MAY", and
"OPTIONAL" in this document are to be interpreted as described in
BCP 14 §RFC2119 §RFC8174 when, and only when, they appear in all
capitals, as shown here.

Widget:
: A JSON object a client and a server keep in step, identified by its "id".

Collection:
: The resource whose representation lists every widget of one owner.

Change token:
: An opaque string a server issues with a listing, naming the point in the
  collection's history that listing reflects.

## 3. Widget Resources

### 3.1. Widget Representation

A widget is a JSON object with three members. "id" is a string the server
assigns and never changes. "name" is a string the client chooses.
"revision" is an integer the server increments on every change. A client
MUST ignore members it does not recognize.

### 3.2. Entity Tags

A server MUST send a strong entity tag (§RFC9110.8.8.3) in every response
that carries a widget representation. The entity tag MUST change whenever
any member of the widget changes, and MUST NOT change otherwise.

## 4. Synchronization

### 4.1. Listing Changes

A client lists changes with a GET request to the collection, passing the
change token it last received in the "since" query parameter. A client
that holds no token omits the parameter and receives every widget.

A server responds with 200 (OK) and a JSON object with two members:
"changes", the widgets changed since the token, and "token", the change
token for this listing. A server that no longer holds the history a token
names MUST respond with 410 (Gone) (§RFC9110.15.5.11); the client then
lists the collection again without "since".

### 4.2. Conditional Update

A client updates a widget with a PUT request that MUST carry an If-Match
header field (§RFC9110.13.1.1) holding the entity tag of the widget it
last received. A server MUST respond to a PUT without If-Match with
428 (Precondition Required) (§RFC6585) and leave the widget unchanged.

A server MUST evaluate the precondition as §RFC9110.13.2 specifies. When
it is false, the server MUST respond with 412 (Precondition Failed)
(§RFC9110.15.5.13) and leave the widget unchanged. When it is true, the
server applies the update, increments "revision", and responds with
200 (OK), the new representation, and its new entity tag.

### 4.3. Conflict Reporting

A 412 response SHOULD carry a problem details object (§RFC9457.3) whose
"type" (§RFC9457.3.1.1) is the Stale Widget problem type and whose
"current" extension member (§RFC9457.3.2) is the server's widget. A client
that receives one SHOULD merge its change into "current" and retry with
the entity tag the response carries.

## security: Security Considerations

Widget Sync adds no protection of its own to HTTP. A server MUST offer it
only over "https" URIs (§RFC9110.4.2.2), and MUST apply to a change
listing the same access control it applies to a full listing: a change
token is not an authorization.

A change token that encodes a sequence number reveals how often a
collection changes. A server SHOULD make tokens opaque, for example by
encrypting them.

Conditional updates prevent lost updates between cooperating clients.
They do not stop a client that may write a widget from overwriting it,
since that client can always send the current entity tag.

Listing changes from an old token can be expensive. A server MAY bound the
history it keeps and answer 410 (Gone) beyond it, which bounds that cost.

## iana: IANA Considerations

IANA is requested to register the following entry in the "HTTP Problem
Types" registry (§RFC9457.4.2):

Type URI:
: https://iana.org/assignments/http-problem-types#stale-widget

Title:
: Stale Widget

Recommended HTTP status code:
: 412

Reference:
: This document, §draft-ietf-wsync-widget-sync.4.3

--- back

## a: Example Exchange

A client that last received revision 3 renames widget "w1" after another
client already renamed it:

~~~ http-message
PUT /widgets/w1 HTTP/1.1
Host: sync.example
If-Match: "w1-3"
Content-Type: application/json

{"id": "w1", "name": "Blue widget", "revision": 3}

HTTP/1.1 412 Precondition Failed
ETag: "w1-4"
Content-Type: application/problem+json

{"type": "https://iana.org/assignments/http-problem-types#stale-widget",
 "title": "Stale Widget", "status": 412,
 "current": {"id": "w1", "name": "Teal widget", "revision": 4}}
~~~

## acknowledgments: Acknowledgments
{:numbered="false"}

The authors thank the members of the Widget Synchronization working group
for their reviews of early versions of this document.
