import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_input_control_cubit.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_user_cubit.dart';
import 'package:appflowy/plugins/ai_chat/application/document_mention_extractor.dart';
import 'package:appflowy/plugins/ai_chat/presentation/layout_define.dart';
import 'package:appflowy/startup/startup.dart';
import 'package:appflowy/util/theme_extension.dart';
import 'package:appflowy/workspace/application/command_palette/command_palette_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/protobuf.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:extended_text_field/extended_text_field.dart';
import 'package:flowy_infra/file_picker/file_picker_service.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import 'at_mention_text_span.dart';
import 'browse_prompts_button.dart';

typedef OnPromptInputSubmitted = void Function(
  String input,
  PredefinedFormat? predefinedFormat,
  Map<String, dynamic> metadata,
  String? promptId,
);

class DesktopPromptInput extends StatefulWidget {
  const DesktopPromptInput({
    super.key,
    required this.isStreaming,
    required this.textController,
    required this.onStopStreaming,
    required this.onSubmitted,
    required this.selectedSourcesNotifier,
    required this.onUpdateSelectedSources,
    this.hideDecoration = false,
    this.hideFormats = false,
    this.extraBottomActionButton,
  });

  final bool isStreaming;
  final AiPromptInputTextEditingController textController;
  final void Function() onStopStreaming;
  final OnPromptInputSubmitted onSubmitted;
  final ValueNotifier<List<String>> selectedSourcesNotifier;
  final void Function(List<String>) onUpdateSelectedSources;
  final bool hideDecoration;
  final bool hideFormats;
  final Widget? extraBottomActionButton;

  @override
  State<DesktopPromptInput> createState() => _DesktopPromptInputState();
}

class _DesktopPromptInputState extends State<DesktopPromptInput> {
  final textFieldKey = GlobalKey();
  final layerLink = LayerLink();
  final overlayController = OverlayPortalController();
  final inputControlCubit = ChatInputControlCubit();
  final chatUserCubit = ChatUserCubit();
  final focusNode = FocusNode();

  late SendButtonState sendButtonState;
  bool isComposing = false;

  // Bidirectional sync state
  late final DocumentMentionExtractor _mentionExtractor;
  bool _isSyncingFromText = false; // Flag to prevent loop when syncing from text
  bool _isSyncingFromSelector = false; // Flag to prevent loop when syncing from selector

  @override
  void initState() {
    super.initState();

    // Initialize mention extractor
    _mentionExtractor = DocumentMentionExtractor(inputControlCubit);

    widget.textController.addListener(handleTextControllerChanged);
    
    // Listen to selectedSourcesNotifier changes for bidirectional sync
    widget.selectedSourcesNotifier.addListener(_onSelectedSourcesChanged);
    
    // Enable mention tracking in ChatInputControlCubit
    inputControlCubit.enableMentionTracking(widget.textController);
    
    // Refresh views to populate document mapping
    inputControlCubit.refreshViews();
    
    focusNode
      ..addListener(
        () {
          if (!widget.hideDecoration) {
            setState(() {}); // refresh border color
          }
          if (!focusNode.hasFocus) {
            cancelMentionPage(); // hide menu when lost focus
          }
        },
      )
      ..onKeyEvent = handleKeyEvent;

    updateSendButtonState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      focusNode.requestFocus();
      checkForAskingAI();
      // Initial sync of @mentions in text to selector
      _syncMentionsToSelector();
    });
  }

  @override
  void didUpdateWidget(covariant oldWidget) {
    updateSendButtonState();
    super.didUpdateWidget(oldWidget);
  }

  @override
  void dispose() {
    widget.selectedSourcesNotifier.removeListener(_onSelectedSourcesChanged);
    focusNode.dispose();
    widget.textController.removeListener(handleTextControllerChanged);
    inputControlCubit.close();
    chatUserCubit.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider.value(value: inputControlCubit),
        BlocProvider.value(value: chatUserCubit),
      ],
      child: BlocListener<ChatInputControlCubit, ChatInputControlState>(
        listener: (context, state) {
          state.maybeWhen(
            updateSelectedViews: (selectedViews) {
              context
                  .read<AIPromptInputBloc>()
                  .add(AIPromptInputEvent.updateMentionedViews(selectedViews));
            },
            orElse: () {},
          );
        },
        child: OverlayPortal(
          controller: overlayController,
          overlayChildBuilder: (context) {
            return PromptInputMentionPageMenu(
              anchor: PromptInputAnchor(textFieldKey, layerLink),
              textController: widget.textController,
              onPageSelected: handlePageSelected,
            );
          },
          child: DecoratedBox(
            decoration: decoration(context),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                ConstrainedBox(
                  constraints: BoxConstraints(
                    maxHeight:
                        DesktopAIPromptSizes.attachedFilesBarPadding.vertical +
                            DesktopAIPromptSizes.attachedFilesPreviewHeight,
                  ),
                  child: TextFieldTapRegion(
                    child: PromptInputFile(
                      onDeleted: (file) => context
                          .read<AIPromptInputBloc>()
                          .add(AIPromptInputEvent.removeFile(file)),
                    ),
                  ),
                ),
                const VSpace(4.0),
                BlocBuilder<AIPromptInputBloc, AIPromptInputState>(
                  builder: (context, state) {
                    return Stack(
                      children: [
                        ConstrainedBox(
                          constraints: getTextFieldConstraints(
                            state.showPredefinedFormats && !widget.hideFormats,
                          ),
                          child: inputTextField(),
                        ),
                        if (state.showPredefinedFormats && !widget.hideFormats)
                          Positioned.fill(
                            bottom: null,
                            child: TextFieldTapRegion(
                              child: Padding(
                                padding: const EdgeInsetsDirectional.only(
                                  start: 8.0,
                                ),
                                child: ChangeFormatBar(
                                  showImageFormats:
                                      state.modelState.type == AiType.cloud,
                                  predefinedFormat: state.predefinedFormat,
                                  spacing: 4.0,
                                  onSelectPredefinedFormat: (format) =>
                                      context.read<AIPromptInputBloc>().add(
                                            AIPromptInputEvent
                                                .updatePredefinedFormat(format),
                                          ),
                                ),
                              ),
                            ),
                          ),
                        Positioned.fill(
                          top: null,
                          child: TextFieldTapRegion(
                            child: _PromptBottomActions(
                              showPredefinedFormatBar:
                                  state.showPredefinedFormats,
                              showPredefinedFormatButton: !widget.hideFormats,
                              onTogglePredefinedFormatSection: () =>
                                  context.read<AIPromptInputBloc>().add(
                                        AIPromptInputEvent
                                            .toggleShowPredefinedFormat(),
                                      ),
                              onStartMention: startMentionPageFromButton,
                              sendButtonState: sendButtonState,
                              onSendPressed: handleSend,
                              onStopStreaming: widget.onStopStreaming,
                              selectedSourcesNotifier:
                                  widget.selectedSourcesNotifier,
                              onUpdateSelectedSources:
                                  widget.onUpdateSelectedSources,
                              onSelectPrompt: handleOnSelectPrompt,
                              extraBottomActionButton:
                                  widget.extraBottomActionButton,
                            ),
                          ),
                        ),
                      ],
                    );
                  },
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  BoxDecoration decoration(BuildContext context) {
    if (widget.hideDecoration) {
      return BoxDecoration();
    }
    return BoxDecoration(
      color: Theme.of(context).colorScheme.surface,
      border: Border.all(
        color: focusNode.hasFocus
            ? Theme.of(context).colorScheme.primary
            : Theme.of(context).colorScheme.outline,
        width: focusNode.hasFocus ? 1.5 : 1.0,
      ),
      borderRadius: const BorderRadius.all(Radius.circular(12.0)),
    );
  }

  void checkForAskingAI() {
    final paletteBloc = context.read<CommandPaletteBloc?>(),
        paletteState = paletteBloc?.state;
    if (paletteBloc == null || paletteState == null) return;
    final isAskingAI = paletteState.askAI;
    if (!isAskingAI) return;
    paletteBloc.add(CommandPaletteEvent.askedAI());
    final query = paletteState.query ?? '';
    if (query.isEmpty) return;
    final sources = (paletteState.askAISources ?? []).map((e) => e.id).toList();
    final metadata =
        context.read<AIPromptInputBloc?>()?.consumeMetadata() ?? {};
    final promptBloc = context.read<AIPromptInputBloc?>();
    final promptId = promptBloc?.promptId;
    final promptState = promptBloc?.state;
    final predefinedFormat = promptState?.predefinedFormat;
    if (sources.isNotEmpty) {
      widget.onUpdateSelectedSources(sources);
    }
    widget.onSubmitted.call(query, predefinedFormat, metadata, promptId ?? '');
  }

  void startMentionPageFromButton() {
    if (overlayController.isShowing) {
      return;
    }
    if (!focusNode.hasFocus) {
      focusNode.requestFocus();
    }
    
    // Insert @ symbol at current cursor position
    final textController = widget.textController;
    final text = textController.text;
    final selection = textController.selection;
    final cursorPos = selection.baseOffset;
    
    final newText = text.substring(0, cursorPos) + '@' + text.substring(cursorPos);
    textController.value = TextEditingValue(
      text: newText,
      selection: TextSelection.collapsed(
        offset: cursorPos + 1,
      ),
    );
    
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (context.mounted) {
        inputControlCubit.startSearching(widget.textController.value);
        overlayController.show();
      }
    });
  }

  void cancelMentionPage() {
    if (overlayController.isShowing) {
      inputControlCubit.reset();
      overlayController.hide();
    }
  }

  void updateSendButtonState() {
    if (widget.isStreaming) {
      sendButtonState = SendButtonState.streaming;
    } else if (widget.textController.text.trim().isEmpty) {
      sendButtonState = SendButtonState.disabled;
    } else {
      sendButtonState = SendButtonState.enabled;
    }
  }

  void handleSend() {
    if (widget.isStreaming) {
      return;
    }
    String userInput = widget.textController.text.trim();
    userInput = inputControlCubit.formatIntputText(userInput);
    userInput = AiPromptInputTextEditingController.restore(userInput);

    widget.textController.clear();
    if (userInput.isEmpty) {
      return;
    }

    // get the attached files and mentioned pages
    final metadata = context.read<AIPromptInputBloc>().consumeMetadata();

    final bloc = context.read<AIPromptInputBloc>();
    final showPredefinedFormats = bloc.state.showPredefinedFormats;
    final predefinedFormat = bloc.state.predefinedFormat;

    widget.onSubmitted(
      userInput,
      showPredefinedFormats ? predefinedFormat : null,
      metadata,
      bloc.promptId,
    );
  }

  void handleTextControllerChanged() {
    setState(() {
      // update whether send button is clickable
      updateSendButtonState();
      isComposing = !widget.textController.value.composing.isCollapsed;
    });

    if (isComposing) {
      return;
    }

    final textController = widget.textController;
    final textSelection = textController.value.selection;
    final text = textController.text;

    // Check if mention menu is currently showing
    if (overlayController.isShowing) {
      // Menu is showing, handle filtering and cancellation
      if (inputControlCubit.filterStartPosition != -1) {
        _handleMentionFiltering(text, textSelection);
      }
    } else {
      // Menu is not showing, check if we need to show it
      _checkForAtMention(text, textSelection);
    }

    // Sync @mentions from text to document selector (if not already syncing from selector)
    if (!_isSyncingFromSelector) {
      _syncMentionsToSelector();
    }
  }

  /// Sync @mentions from text to document selector
  /// Extracts all @mention document IDs from text and updates selectedSourcesNotifier
  void _syncMentionsToSelector() {
    if (_isSyncingFromSelector) {
      return; // Already syncing, prevent loop
    }

    try {
      _isSyncingFromText = true;

      final text = widget.textController.text;
      final mentions = _mentionExtractor.parseMentions(text);
      
      // Extract document IDs from mentions, filtering out invalid ones
      // Also handle renamed/deleted documents by checking if document still exists
      final documentIdsFromText = <String>[];
      for (final mention in mentions) {
        if (mention.documentId.isEmpty) {
          continue; // Invalid mention (document renamed or deleted)
        }
        // Verify document still exists and name matches
        final documentName = _getDocumentNameById(mention.documentId);
        if (documentName != null && documentName == mention.documentName) {
          documentIdsFromText.add(mention.documentId);
        }
        // If document name doesn't match, it might have been renamed
        // We don't add it to avoid syncing invalid references
      }

      final documentIdsFromTextSet = documentIdsFromText.toSet();

      // Get current selected sources
      final currentSelectedSources = widget.selectedSourcesNotifier.value.toSet();

      // Only update if there's a difference to avoid unnecessary notifications
      if (currentSelectedSources != documentIdsFromTextSet) {
        // Merge: keep documents selected in selector that aren't in text
        // This allows users to manually add documents via selector
        final mergedIds = <String>[];
        
        // Add all document IDs from text
        mergedIds.addAll(documentIdsFromText);
        
        // Add non-document sources or documents manually selected via selector
        for (final id in currentSelectedSources) {
          if (!documentIdsFromTextSet.contains(id)) {
            // Check if this is a document or other source type
            final documentName = _getDocumentNameById(id);
            if (documentName == null) {
              // Not a document ID, keep it (might be a web source or other type)
              mergedIds.add(id);
            } else {
              // It's a document manually added via selector, keep it
              mergedIds.add(id);
            }
          }
        }

        // Update selectedSourcesNotifier (this will trigger _onSelectedSourcesChanged,
        // but _isSyncingFromText flag prevents it from modifying text)
        widget.selectedSourcesNotifier.value = mergedIds;
      }
    } finally {
      _isSyncingFromText = false;
    }
  }

  /// Handle changes to selectedSourcesNotifier
  /// Removes @mentions from text when documents are removed from selector
  void _onSelectedSourcesChanged() {
    if (_isSyncingFromText) {
      return; // Ignore changes that we triggered ourselves
    }

    try {
      _isSyncingFromSelector = true;

      final currentSelectedIds = widget.selectedSourcesNotifier.value.toSet();
      final text = widget.textController.text;
      final mentions = _mentionExtractor.parseMentions(text);

      // Find mentions whose document IDs are no longer in selectedSources
      final mentionsToRemove = mentions.where((mention) {
        if (mention.documentId.isEmpty) {
          return false; // Invalid mention, skip
        }
        return !currentSelectedIds.contains(mention.documentId);
      }).toList();

      // Remove mentions from text if they were removed from selector
      String currentText = text;
      int offset = 0; // Track cumulative offset changes for cursor position
      
      if (mentionsToRemove.isNotEmpty) {
        // Sort mentions by position in reverse order to remove from end to start
        // This preserves positions when removing multiple mentions
        final sortedMentions = List<DocumentMention>.from(mentionsToRemove)
          ..sort((a, b) => b.startPosition.compareTo(a.startPosition));

        for (final mention in sortedMentions) {
          // Adjust mention positions based on previous removals
          final adjustedStartPos = mention.startPosition + offset;
          final adjustedEndPos = mention.endPosition + offset;
          
          // Remove the mention text from the text
          final beforeRemoval = currentText.substring(0, adjustedStartPos);
          final afterRemoval = currentText.substring(adjustedEndPos);
          currentText = beforeRemoval + afterRemoval;

          // Update offset for cursor position
          offset -= mention.endPosition - mention.startPosition;
        }

        // Update text controller with removed mentions
        final currentSelection = widget.textController.selection;
        final newCursorPosition = (currentSelection.baseOffset + offset).clamp(0, currentText.length);
        
        widget.textController.value = TextEditingValue(
          text: currentText,
          selection: TextSelection.collapsed(offset: newCursorPosition),
        );
      }

      // Also add @mentions for documents that were added via selector but not in text
      // Requirement 5: "IF 在文档选择框中手动添加文档时 THEN 系统 SHALL 同时在输入框中添加 @文档名称 标记（如果不存在）"
      final currentSelectedIdsList = widget.selectedSourcesNotifier.value;
      // Use currentText (which may have been modified by removals) instead of original text
      final documentIdsFromText = _mentionExtractor.extractDocumentIds(currentText).toSet();
      
      // Find documents added via selector that aren't in text
      final documentsToAdd = <String>[];
      for (final selectedId in currentSelectedIdsList) {
        // Check if this is a document ID (not a web source)
        final documentName = _getDocumentNameById(selectedId);
        if (documentName != null && !documentIdsFromText.contains(selectedId)) {
          documentsToAdd.add(selectedId);
        }
      }
      
      // Add mentions to text for newly selected documents
      if (documentsToAdd.isNotEmpty) {
        String newText = currentText;
        for (final documentId in documentsToAdd) {
          final documentName = _getDocumentNameById(documentId);
          if (documentName != null) {
            final mentionText = '@$documentName';
            // Add separator if needed
            if (newText.isNotEmpty && !newText.endsWith(' ') && !newText.endsWith('\n')) {
              newText += ' ';
            }
            newText += mentionText;
          }
        }
        
        if (newText != currentText) {
          widget.textController.value = TextEditingValue(
            text: newText,
            selection: TextSelection.collapsed(offset: newText.length),
          );
        }
      }
    } finally {
      _isSyncingFromSelector = false;
    }
  }

  /// Get document name by ID, return null if not found (might be web source or deleted)
  String? _getDocumentNameById(String documentId) {
    try {
      final view = inputControlCubit.allViews.firstWhere(
        (v) => v.id == documentId,
        orElse: () => ViewPB(),
      );
      
      if (view.id.isEmpty) {
        return null; // Document not found (might be deleted or renamed)
      }
      
      return view.name.isNotEmpty
          ? view.name
          : LocaleKeys.document_title_placeholder.tr();
    } catch (e) {
      return null; // Error finding document
    }
  }

  /// Check if @ symbol is typed and show mention menu
  void _checkForAtMention(String text, TextSelection selection) {
    final cursorPos = selection.baseOffset;
    
    // Check if cursor is in a valid position
    if (cursorPos <= 0 || cursorPos > text.length) {
      return;
    }
    
    // Check if character before cursor is @
    final charBeforeCursor = text[cursorPos - 1];
    if (charBeforeCursor == '@') {
      // @ symbol detected, show mention menu
      // ChatInputControlCubit._onTextChanged() will handle the state tracking,
      // we just need to show the overlay menu if it's not already showing
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (context.mounted && !overlayController.isShowing) {
          // Ensure cubit is ready for searching
          if (inputControlCubit.filterStartPosition == -1) {
            inputControlCubit.startSearching(widget.textController.value);
          }
          overlayController.show();
        }
      });
    }
  }

  /// Handle mention filtering when menu is showing
  void _handleMentionFiltering(String text, TextSelection selection) {
    // Handle cases where mention is cancelled
    final isSelectingMultipleCharacters = !selection.isCollapsed;
    final isCaretBeforeStartOfRange =
        selection.baseOffset < inputControlCubit.filterStartPosition;
    final isCaretAfterEndOfRange =
        selection.baseOffset > inputControlCubit.filterEndPosition;
    final isTextSame = inputControlCubit.inputText == text;

    // Cancel if selecting multiple characters or cursor moved outside mention range
    if (isSelectingMultipleCharacters ||
        (isTextSame && (isCaretBeforeStartOfRange || isCaretAfterEndOfRange))) {
      cancelMentionPage();
      return;
    }

    final previousLength = inputControlCubit.inputText.characters.length;
    final currentLength = text.characters.length;

    // Cancel if @ symbol is deleted
    if (previousLength != currentLength && isCaretBeforeStartOfRange) {
      cancelMentionPage();
      return;
    }

    // Update filter when text changes
    if (previousLength != currentLength) {
      final diff = currentLength - previousLength;
      final newEndPosition = inputControlCubit.filterEndPosition + diff;
      
      // Ensure positions are valid
      if (newEndPosition < inputControlCubit.filterStartPosition ||
          inputControlCubit.filterStartPosition < 0 ||
          newEndPosition > text.length) {
        cancelMentionPage();
        return;
      }
      
      final newFilter = text.substring(
        inputControlCubit.filterStartPosition,
        newEndPosition,
      );
      inputControlCubit.updateFilter(
        text,
        newFilter,
        newEndPosition: newEndPosition,
      );
    } else if (!isTextSame) {
      // Text changed without length change (e.g., paste)
      if (inputControlCubit.filterEndPosition > text.length ||
          inputControlCubit.filterStartPosition < 0) {
        cancelMentionPage();
        return;
      }
      
      final newFilter = text.substring(
        inputControlCubit.filterStartPosition,
        inputControlCubit.filterEndPosition,
      );
      inputControlCubit.updateFilter(text, newFilter);
    }
  }

  KeyEventResult handleKeyEvent(FocusNode node, KeyEvent event) {
    if (event is KeyDownEvent &&
        event.logicalKey == LogicalKeyboardKey.escape) {
      if (overlayController.isShowing) {
        cancelMentionPage();
        return KeyEventResult.handled;
      }
      node.unfocus();
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  void handlePageSelected(ViewPB view) {
    // Get document name (use placeholder if empty)
    final documentName = view.name.isNotEmpty
        ? view.name
        : LocaleKeys.document_title_placeholder.tr();
    
    // Replace the range from @ to cursor with @documentName
    // 🔧 修复问题3：自动在文档名后添加空格，使@文档名称自动变为蓝色
    final mentionText = '@$documentName ';
    
    // Ensure positions are valid
    final startPos = inputControlCubit.filterStartPosition;
    final endPos = inputControlCubit.filterEndPosition;
    
    if (startPos < 0 || endPos < startPos || endPos > widget.textController.text.length) {
      // Invalid positions, just insert at cursor
      final cursorPos = widget.textController.selection.baseOffset;
      final text = widget.textController.text;
      final newText = text.substring(0, cursorPos) + mentionText + text.substring(cursorPos);
      
      widget.textController.value = TextEditingValue(
        text: newText,
        selection: TextSelection.collapsed(
          offset: cursorPos + mentionText.length,
        ),
      );
    } else {
      // Replace the range from @ symbol to cursor
      final newText = widget.textController.text.replaceRange(
        startPos - 1, // Include the @ symbol
        endPos,
        mentionText,
      );
      
      widget.textController.value = TextEditingValue(
        text: newText,
        selection: TextSelection.collapsed(
          offset: startPos - 1 + mentionText.length,
          affinity: TextAffinity.upstream,
        ),
      );
    }

    inputControlCubit.selectPage(view);
    overlayController.hide();
  }

  Widget inputTextField() {
    return Shortcuts(
      shortcuts: buildShortcuts(),
      child: Actions(
        actions: buildActions(),
        child: CompositedTransformTarget(
          link: layerLink,
          child: BlocBuilder<AIPromptInputBloc, AIPromptInputState>(
            builder: (context, state) {
              // Use ValueListenableBuilder to rebuild text field when text changes
              // This ensures that AtMentionTextSpanBuilder is recreated and 
              // finishText() is called again to re-validate document mentions
              return ValueListenableBuilder<TextEditingValue>(
                valueListenable: widget.textController,
                builder: (context, textValue, child) {
                  Widget textField = PromptInputTextField(
                    key: textFieldKey,
                    editable: state.modelState.isEditable,
                    cubit: inputControlCubit,
                    textController: widget.textController,
                    textFieldFocusNode: focusNode,
                    contentPadding:
                        calculateContentPadding(state.showPredefinedFormats),
                    hintText: state.modelState.hintText,
                  );

                  if (state.modelState.tooltip != null) {
                    textField = FlowyTooltip(
                      message: state.modelState.tooltip!,
                      child: textField,
                    );
                  }

                  return textField;
                },
              );
            },
          ),
        ),
      ),
    );
  }

  BoxConstraints getTextFieldConstraints(bool showPredefinedFormats) {
    double minHeight = DesktopAIPromptSizes.textFieldMinHeight +
        DesktopAIPromptSizes.actionBarSendButtonSize +
        DesktopAIChatSizes.inputActionBarMargin.vertical;
    double maxHeight = 300;
    if (showPredefinedFormats) {
      minHeight += DesktopAIPromptSizes.predefinedFormatButtonHeight;
      maxHeight += DesktopAIPromptSizes.predefinedFormatButtonHeight;
    }
    return BoxConstraints(minHeight: minHeight, maxHeight: maxHeight);
  }

  EdgeInsetsGeometry calculateContentPadding(bool showPredefinedFormats) {
    final top = showPredefinedFormats
        ? DesktopAIPromptSizes.predefinedFormatButtonHeight
        : 0.0;
    final bottom = DesktopAIPromptSizes.actionBarSendButtonSize +
        DesktopAIChatSizes.inputActionBarMargin.vertical;

    return DesktopAIPromptSizes.textFieldContentPadding
        .add(EdgeInsets.only(top: top, bottom: bottom));
  }

  Map<ShortcutActivator, Intent> buildShortcuts() {
    if (isComposing) {
      return const {};
    }

    return const {
      SingleActivator(LogicalKeyboardKey.arrowUp): _FocusPreviousItemIntent(),
      SingleActivator(LogicalKeyboardKey.arrowDown): _FocusNextItemIntent(),
      SingleActivator(LogicalKeyboardKey.escape): _CancelMentionPageIntent(),
      SingleActivator(LogicalKeyboardKey.enter): _SubmitOrMentionPageIntent(),
    };
  }

  Map<Type, Action<Intent>> buildActions() {
    return {
      _FocusPreviousItemIntent: CallbackAction<_FocusPreviousItemIntent>(
        onInvoke: (intent) {
          inputControlCubit.updateSelectionUp();
          return;
        },
      ),
      _FocusNextItemIntent: CallbackAction<_FocusNextItemIntent>(
        onInvoke: (intent) {
          inputControlCubit.updateSelectionDown();
          return;
        },
      ),
      _CancelMentionPageIntent: CallbackAction<_CancelMentionPageIntent>(
        onInvoke: (intent) {
          cancelMentionPage();
          return;
        },
      ),
      _SubmitOrMentionPageIntent: CallbackAction<_SubmitOrMentionPageIntent>(
        onInvoke: (intent) {
          if (overlayController.isShowing) {
            inputControlCubit.state.maybeWhen(
              ready: (visibleViews, focusedViewIndex) {
                if (focusedViewIndex != -1 &&
                    focusedViewIndex < visibleViews.length) {
                  handlePageSelected(visibleViews[focusedViewIndex]);
                }
              },
              orElse: () {},
            );
          } else {
            handleSend();
          }
          return;
        },
      ),
    };
  }

  void handleOnSelectPrompt(AiPrompt prompt) {
    final bloc = context.read<AIPromptInputBloc>();
    bloc
      ..add(AIPromptInputEvent.updateMentionedViews([]))
      ..add(AIPromptInputEvent.updatePromptId(prompt.id));

    final content = AiPromptInputTextEditingController.replace(prompt.content);

    widget.textController.value = TextEditingValue(
      text: content,
      selection: TextSelection.collapsed(
        offset: content.length,
      ),
    );

    if (bloc.state.showPredefinedFormats) {
      bloc.add(
        AIPromptInputEvent.toggleShowPredefinedFormat(),
      );
    }
  }
}

class _SubmitOrMentionPageIntent extends Intent {
  const _SubmitOrMentionPageIntent();
}

class _CancelMentionPageIntent extends Intent {
  const _CancelMentionPageIntent();
}

class _FocusPreviousItemIntent extends Intent {
  const _FocusPreviousItemIntent();
}

class _FocusNextItemIntent extends Intent {
  const _FocusNextItemIntent();
}

class PromptInputTextField extends StatelessWidget {
  const PromptInputTextField({
    super.key,
    required this.editable,
    required this.cubit,
    required this.textController,
    required this.textFieldFocusNode,
    required this.contentPadding,
    this.hintText = "",
  });

  final ChatInputControlCubit cubit;
  final TextEditingController textController;
  final FocusNode textFieldFocusNode;
  final EdgeInsetsGeometry contentPadding;
  final bool editable;
  final String hintText;

  @override
  Widget build(BuildContext context) {
    final theme = AppFlowyTheme.of(context);

    return ExtendedTextField(
      controller: textController,
      focusNode: textFieldFocusNode,
      readOnly: !editable,
      enabled: editable,
      decoration: InputDecoration(
        border: InputBorder.none,
        enabledBorder: InputBorder.none,
        focusedBorder: InputBorder.none,
        contentPadding: contentPadding,
        hintText: hintText,
        hintStyle: inputHintTextStyle(context),
        isCollapsed: true,
        isDense: true,
      ),
      keyboardType: TextInputType.multiline,
      textCapitalization: TextCapitalization.sentences,
      minLines: 1,
      maxLines: null,
      style: theme.textStyle.body.standard(
        color: theme.textColorScheme.primary,
      ),
      specialTextSpanBuilder: AtMentionTextSpanBuilder(
        inputControlCubit: cubit,
        atMentionTextStyle: theme.textStyle.body.standard().copyWith(
          color: Colors.blue,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }

  TextStyle? inputHintTextStyle(BuildContext context) {
    return AppFlowyTheme.of(context).textStyle.body.standard(
          color: Theme.of(context).isLightMode
              ? const Color(0xFFBDC2C8)
              : const Color(0xFF3C3E51),
        );
  }
}

class _PromptBottomActions extends StatelessWidget {
  const _PromptBottomActions({
    required this.sendButtonState,
    required this.showPredefinedFormatBar,
    required this.showPredefinedFormatButton,
    required this.onTogglePredefinedFormatSection,
    required this.onStartMention,
    required this.onSendPressed,
    required this.onStopStreaming,
    required this.selectedSourcesNotifier,
    required this.onUpdateSelectedSources,
    required this.onSelectPrompt,
    this.extraBottomActionButton,
  });

  final bool showPredefinedFormatBar;
  final bool showPredefinedFormatButton;
  final void Function() onTogglePredefinedFormatSection;
  final void Function() onStartMention;
  final SendButtonState sendButtonState;
  final void Function() onSendPressed;
  final void Function() onStopStreaming;
  final ValueNotifier<List<String>> selectedSourcesNotifier;
  final void Function(List<String>) onUpdateSelectedSources;
  final void Function(AiPrompt) onSelectPrompt;
  final Widget? extraBottomActionButton;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: DesktopAIPromptSizes.actionBarSendButtonSize,
      margin: DesktopAIChatSizes.inputActionBarMargin,
      child: BlocBuilder<AIPromptInputBloc, AIPromptInputState>(
        builder: (context, state) {
          return Row(
            spacing: DesktopAIChatSizes.inputActionBarButtonSpacing,
            children: [
              if (showPredefinedFormatButton) _predefinedFormatButton(),
              _selectModelButton(context),
              _buildBrowsePromptsButton(),

              const Spacer(),

              if (context.read<ChatUserCubit>().supportSelectSource())
                _selectSourcesButton(),

              if (extraBottomActionButton != null) extraBottomActionButton!,
              // _mentionButton(context),
              if (state.supportChatWithFile) _attachmentButton(context),
              _sendButton(),
            ],
          );
        },
      ),
    );
  }

  Widget _predefinedFormatButton() {
    return PromptInputDesktopToggleFormatButton(
      showFormatBar: showPredefinedFormatBar,
      onTap: onTogglePredefinedFormatSection,
    );
  }

  Widget _selectSourcesButton() {
    return PromptInputDesktopSelectSourcesButton(
      onUpdateSelectedSources: onUpdateSelectedSources,
      selectedSourcesNotifier: selectedSourcesNotifier,
    );
  }

  Widget _selectModelButton(BuildContext context) {
    return SelectModelMenu(
      aiModelStateNotifier:
          context.read<AIPromptInputBloc>().aiModelStateNotifier,
    );
  }

  Widget _buildBrowsePromptsButton() {
    return BrowsePromptsButton(
      onSelectPrompt: onSelectPrompt,
    );
  }

  // Widget _mentionButton(BuildContext context) {
  //   return PromptInputMentionButton(
  //     iconSize: DesktopAIPromptSizes.actionBarIconSize,
  //     buttonSize: DesktopAIPromptSizes.actionBarButtonSize,
  //     onTap: onStartMention,
  //   );
  // }

  Widget _attachmentButton(BuildContext context) {
    return PromptInputAttachmentButton(
      onTap: () async {
        final path = await getIt<FilePickerService>().pickFiles(
          dialogTitle: '',
          type: FileType.custom,
          allowedExtensions: ["pdf", "txt", "md"],
        );

        if (path == null) {
          return;
        }

        for (final file in path.files) {
          if (file.path != null && context.mounted) {
            context
                .read<AIPromptInputBloc>()
                .add(AIPromptInputEvent.attachFile(file.path!, file.name));
          }
        }
      },
    );
  }

  Widget _sendButton() {
    return PromptInputSendButton(
      state: sendButtonState,
      onSendPressed: onSendPressed,
      onStopStreaming: onStopStreaming,
    );
  }
}
