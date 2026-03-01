# Feature Specification: Voice Conversation Transcription

**Feature Branch**: `001-voice-transcription`
**Created**: 2026-03-01
**Status**: Draft
**Input**: User description: "Desktop app that helps users analyze voice conversations and transcribe them. Upload existing audio files. Summaries and transcriptions exportable to markdown or PDF. Sleek UI with chat-like conversation organization."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Upload and Transcribe Audio (Priority: P1)

A user opens Yap and uploads an existing audio file (e.g., a
meeting recording, voice memo, or interview). The app processes
the file locally, produces a full text transcription, and
generates a short summary of the conversation. The result is
saved and immediately viewable.

**Why this priority**: This is the core value proposition.
Without upload and transcription, no other feature has meaning.
This alone delivers a usable MVP.

**Independent Test**: Can be fully tested by uploading a short
audio file and verifying that a transcription with summary
appears. Delivers immediate value as a standalone transcription
tool.

**Acceptance Scenarios**:

1. **Given** the app is open, **When** the user uploads an audio
   file (e.g., .mp3, .wav, .m4a, .ogg, .webm), **Then** the
   app begins transcribing and shows a progress indicator.
2. **Given** an audio file is being transcribed, **When**
   processing completes, **Then** the user sees a full text
   transcription with a 1-3 sentence summary at the top.
3. **Given** a completed transcription, **When** the user views
   it, **Then** they see timestamped text segments with speaker
   labels (when distinguishable) and the summary.
4. **Given** the user has downloaded a transcription model and
   has no internet connection, **When** they upload and process
   an audio file, **Then** the transcription completes
   successfully using local processing.
6. **Given** the app is open, **When** the user drags an audio
   file onto the app window, **Then** the file is accepted and
   transcription begins (drag-and-drop support).

---

### User Story 2 - Browse and Manage Conversations (Priority: P2)

A user with multiple transcribed conversations opens Yap and sees
a sidebar listing all their conversations, organized
chronologically (newest first), similar to how ChatGPT, Claude,
or Granola display conversation history. Each entry shows the
summary or title. The user can select any conversation to view
its full transcription, rename it, or delete it.

**Why this priority**: Organization is essential once a user has
more than a handful of transcriptions. Without it, the app
becomes unusable at scale. This is the second most critical
feature after transcription itself.

**Independent Test**: Can be tested by uploading several audio
files and verifying the resulting conversations appear in the
sidebar, can be selected, renamed, searched, and deleted.

**Acceptance Scenarios**:

1. **Given** the user has 5 transcribed conversations, **When**
   they open the app, **Then** a sidebar displays all 5
   conversations listed by date with their summary or title
   visible.
2. **Given** a conversation is listed in the sidebar, **When**
   the user clicks on it, **Then** the main content area
   displays the full transcription and summary.
3. **Given** a conversation exists, **When** the user renames it,
   **Then** the new name appears in the sidebar immediately.
4. **Given** a conversation exists, **When** the user deletes it,
   **Then** it is removed from the sidebar and its data is
   permanently deleted from local storage after confirmation.
5. **Given** the user has many conversations, **When** they type
   in the search bar, **Then** conversations are filtered by
   title, summary content, or transcription text.

---

### User Story 3 - Export Conversations (Priority: P3)

A user selects a conversation and exports it as either a Markdown
file or a PDF file. The exported file contains the conversation
title, summary, and full transcription, formatted cleanly.

**Why this priority**: Export is a key differentiator and user
request, but the app is fully usable without it. It builds on
the transcription and organization features.

**Independent Test**: Can be tested by exporting a conversation
to each format and verifying the output file contains the
expected content and formatting.

**Acceptance Scenarios**:

1. **Given** a transcribed conversation is selected, **When**
   the user clicks "Export" and chooses "Markdown", **Then**
   a `.md` file is saved to their chosen location containing
   the title, summary, and full transcription.
2. **Given** a transcribed conversation is selected, **When**
   the user clicks "Export" and chooses "PDF", **Then** a
   `.pdf` file is saved to their chosen location with clean
   formatting, title, summary, and full transcription.
3. **Given** the user triggers export, **When** they are
   presented with the format choice, **Then** a native file
   save dialog appears letting them choose the destination
   and filename.
4. **Given** a conversation with speaker labels, **When**
   exported, **Then** the speaker labels are preserved in the
   output format.

---

### Edge Cases

- What happens when the user uploads an unsupported file format?
  The app MUST show a clear error message listing supported
  formats.
- What happens when an audio file is very long (e.g., 2+ hours)?
  The app MUST handle long files without crashing or running out
  of memory, processing in chunks if needed. For files exceeding
  4 hours, the app MUST display a warning that processing may
  take a long time, but still allow the user to proceed.
- What happens when the user closes the app during transcription?
  The app MUST either complete the transcription in the
  background or resume it on next launch.
- What happens when disk space is low? The app MUST warn the
  user before processing if available space is critically low.
- What happens when audio quality is poor (background noise,
  low volume)? The transcription MUST still produce output,
  with low-confidence sections indicated to the user.
- What happens if the user tries to export a conversation that
  is still being transcribed? The export option MUST be
  disabled or the user informed to wait.
- What happens when a user uploads a duplicate file? The app
  MUST create a separate conversation entry (not silently
  deduplicate).
- What happens when the uploaded file contains no speech
  (e.g., music, silence)? The app MUST inform the user that
  no speech was detected.
- What happens when no transcription model is downloaded? The
  app MUST direct the user to the setup wizard or settings to
  download a model before allowing uploads.
- What happens on first launch? The app MUST present the setup
  wizard before allowing any uploads.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept audio file uploads via file
  picker and drag-and-drop.
- **FR-002**: System MUST support common audio formats: .mp3,
  .wav, .m4a, .ogg, and .webm at minimum.
- **FR-003**: System MUST transcribe audio to text locally using
  sherpa-onnx (via sherpa-rs) with Whisper ONNX models. All
  transcription and speaker diarization runs on-device with no
  cloud dependency.
- **FR-004**: System MUST generate a 1-3 sentence summary of
  each transcribed conversation automatically.
- **FR-005**: System MUST display conversations in a sidebar
  list, ordered chronologically (newest first), showing the
  title/summary for each entry.
- **FR-006**: System MUST allow users to select a conversation
  from the sidebar to view its full transcription and summary
  in a main content area.
- **FR-007**: System MUST allow users to rename and delete
  conversations.
- **FR-008**: System MUST support searching conversations by
  title, summary, or transcription content.
- **FR-009**: System MUST export conversations to Markdown
  format (.md files).
- **FR-010**: System MUST export conversations to PDF format
  (.pdf files).
- **FR-011**: System MUST use native file dialogs for both
  upload and export operations.
- **FR-012**: System MUST persist all conversation data locally
  using platform-appropriate storage.
- **FR-013**: System MUST display transcription progress (e.g.,
  percentage or time remaining) while processing.
- **FR-015**: System MUST present a first-run setup wizard that
  guides the user through selecting and downloading a local
  transcription model (e.g., Whisper base, small, or medium).
- **FR-016**: System MUST provide a settings screen where the
  user can change their model selection and manage downloaded
  models after initial setup.
- **FR-018**: System MUST retain the original uploaded audio file
  locally and allow the user to replay it within the app.
- **FR-014**: System MUST present a sleek, modern UI drawing
  inspiration from the Figma designs (see Design Reference
  below). The original designs target web; the desktop app
  MUST adapt layout and interactions to feel native on desktop.

### Design Reference

**Source**: [Figma — YAP MVP](https://www.figma.com/design/kqfgztNJlbDTg75bCrcgSY/YAP-MVP?node-id=42-681&p=f&m=dev)

**Important**: These designs were created for a web app. They
serve as *inspiration* for visual direction and information
architecture. The desktop app layout may differ to feel native
on each platform.

**Key design patterns to carry forward**:

- **Dark theme**: Dark background with light text throughout.
  Green accent color (#00C853-ish) for primary actions and
  status indicators.
- **Session concept**: A "Session" (e.g., "Nurture Therapy")
  groups related audio uploads together. Each session contains
  multiple transcriptions listed in a table with ID, summary
  preview, status, date, and actions.
- **History panel**: A right-side panel lists sessions for
  quick navigation (titled "History"). Each entry shows the
  session name with an icon indicating type (audio or notes).
- **Transcript vs Insights toggle**: The detail view has a
  toggle between "Transcript" (full timestamped text) and
  "Insights" (key points as bullet list). This maps to our
  transcription and summary features.
- **Speaker role assignment**: Users can assign names to
  detected speakers ("Person 1", "Person 2") via editable
  name fields in the "Assign Roles" section.
- **Upload flow**: Dedicated upload page with file picker,
  supported format info, and Cancel/Upload buttons. After
  upload, a row appears with "Analyzing" status and spinner.
- **Empty state**: When no transcriptions exist, a dashed
  border area with "Upload audio to generate insights &
  transcript" prompt.
- **Top navigation**: Breadcrumb path (Home / Session / Audio)
  plus nav items (Sessions, Transcribe, Settings).
- **Status badges**: Color-coded status labels — green
  "Completed", yellow/orange "Analyzing".
- **Actions per transcription**: Icons for insights, archive,
  and navigation (chevron to open detail).

**Desktop adaptations to consider**:

- The web History panel (right-side) may work better as a
  left sidebar on desktop, aligning with native app patterns.
- Native window chrome, menus, and keyboard shortcuts.
- Drag-and-drop from OS file manager onto the app window.
- Native file dialogs for upload and export.

### Key Entities

- **Session**: A grouping of related conversations/uploads
  (e.g., "Nurture Therapy", "Team Standup"). Attributes:
  title (user-editable), creation date, description.
- **Conversation**: A single uploaded audio file within a
  session. Attributes: sequence number within session,
  title (user-editable, defaults to filename), creation date,
  duration, source audio file (retained for playback),
  processing status (uploading, analyzing, completed, error).
- **Transcription**: The text output of a conversation.
  Attributes: full text content, speaker labels (when
  detectable), timestamps per segment, confidence indicators.
- **Summary / Insights**: Auto-generated key points from a
  conversation. Attributes: bullet-point list of key points,
  generation date. Displayed in an "Insights" view alongside
  the full transcript.

## Clarifications

### Session 2026-03-01

- Q: What languages does transcription support? → A: Language support is determined by the user's chosen transcription backend. The app delegates to the selected model.
- Q: Does the app keep the original audio file after transcription? → A: Yes. The app retains the original audio so users can replay within the app.
- Q: Is there a maximum audio file duration? → A: Soft limit at 4 hours. The app warns the user but still allows processing.
- Q: How does the user configure their transcription model? → A: First-run setup wizard guides the user through selecting and downloading a local Whisper model, editable later in settings.
- Design review: Figma designs reviewed via MCP. Key patterns extracted (dark theme, session grouping, transcript/insights toggle, speaker roles, status badges). Designs are web-origin — desktop app will adapt to native patterns.

## Assumptions

- **Audio source**: Users upload pre-existing audio files. Live
  microphone recording is not in scope for this feature.
- **Supported formats**: The app covers the most common audio
  formats (.mp3, .wav, .m4a, .ogg, .webm). Exotic or
  proprietary formats are out of scope.
- **Speaker diarization**: Basic speaker labeling is included
  when audio quality permits, but perfect speaker identification
  is not guaranteed.
- **Transcription backend**: All transcription runs locally via
  sherpa-onnx (Whisper ONNX models for STT, pyannote/3dspeaker
  for diarization). No cloud backends. Fully offline.
- **Language support**: Determined by the Whisper model. The app
  does not constrain language — it surfaces whatever the model
  returns.
- **Summary generation**: Summaries are generated locally from
  the transcription output.
- **Design references**: UI draws inspiration from the Figma
  designs (originally web-targeted). The desktop app adapts
  layout and interactions to feel native on each platform.
  See Design Reference section for detailed patterns.
- **Conversation organization**: The sidebar layout follows the
  established pattern from ChatGPT, Claude, and Granola — a
  left sidebar with conversation list and a main content area
  on the right.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can upload an audio file and see a completed
  transcription within 2x the audio duration (e.g., a 5-minute
  file completes in under 10 minutes).
- **SC-002**: Users can find any past conversation within 10
  seconds using search or scrolling.
- **SC-003**: Exported Markdown and PDF files open correctly
  in standard viewers and contain the full conversation content
  (title, summary, transcription).
- **SC-004**: The app launches and is ready to accept uploads
  in under 2 seconds on supported hardware.
- **SC-005**: When a local model is selected, all core features
  (upload, transcribe, summarize, browse, export) work fully
  offline with no network connection.
- **SC-006**: The app runs on macOS, Windows, and Linux with
  consistent functionality and appearance.
- **SC-007**: 90% of users can complete their first
  upload-transcribe-export workflow without external guidance.
