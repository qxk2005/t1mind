import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/view/prelude.dart';
import 'package:appflowy/workspace/application/view/view_ext.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/view.pb.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:bloc/bloc.dart';
import 'package:collection/collection.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'chat_input_control_cubit.freezed.dart';

class ChatInputControlCubit extends Cubit<ChatInputControlState> {
  ChatInputControlCubit() : super(const ChatInputControlState.loading());

  final List<ViewPB> allViews = [];
  final List<String> selectedViewIds = [];

  /// used when mentioning a page
  ///
  /// the text position after the @ character
  int _filterStartPosition = -1;

  /// used when mentioning a page
  ///
  /// the text position after the @ character, at the end of the filter
  int _filterEndPosition = -1;

  /// used when mentioning a page
  ///
  /// the entire string input in the prompt
  String _inputText = "";

  /// used when mentioning a page
  ///
  /// the current filtering text, after the @ characater
  String _filter = "";

  /// Tracks whether @mention mode is currently active
  bool _isAtMentionMode = false;

  /// Tracks the position of the @ symbol in the text
  int _atSymbolPosition = -1;

  /// Reference to the TextEditingController for mention tracking
  TextEditingController? _textController;

  String get inputText => _inputText;
  int get filterStartPosition => _filterStartPosition;
  int get filterEndPosition => _filterEndPosition;
  bool get isAtMentionMode => _isAtMentionMode;
  int get atSymbolPosition => _atSymbolPosition;

  void refreshViews() async {
    final newViews = await ViewBackendService.getAllViews().fold(
      (result) {
        return result.items
            .where(
              (v) =>
                  !v.isSpace &&
                  v.layout.isDocumentView &&
                  v.parentViewId != v.id,
            )
            .toList();
      },
      (err) {
        Log.error(err);
        return <ViewPB>[];
      },
    );
    allViews
      ..clear()
      ..addAll(newViews);

    // update visible views
    newViews.retainWhere((v) => !selectedViewIds.contains(v.id));
    if (_filter.isNotEmpty) {
      newViews.retainWhere(
        (v) {
          final nonEmptyName = v.name.isEmpty
              ? LocaleKeys.document_title_placeholder.tr()
              : v.name;
          return nonEmptyName.toLowerCase().contains(_filter);
        },
      );
    }
    final focusedViewIndex = newViews.isEmpty ? -1 : 0;
    emit(
      ChatInputControlState.ready(
        visibleViews: newViews,
        focusedViewIndex: focusedViewIndex,
      ),
    );
  }

  void startSearching(TextEditingValue textEditingValue) {
    _filterStartPosition =
        _filterEndPosition = textEditingValue.selection.baseOffset;
    _filter = "";
    _inputText = textEditingValue.text;
    state.maybeMap(
      ready: (readyState) {
        emit(
          readyState.copyWith(
            visibleViews: allViews,
            focusedViewIndex: allViews.isEmpty ? -1 : 0,
          ),
        );
      },
      orElse: () {},
    );
  }

  void reset() {
    _filterStartPosition = _filterEndPosition = -1;
    _filter = _inputText = "";
    _cancelAtMention();
    state.maybeMap(
      ready: (readyState) {
        emit(
          readyState.copyWith(
            visibleViews: allViews,
            focusedViewIndex: allViews.isEmpty ? -1 : 0,
          ),
        );
      },
      orElse: () {},
    );
  }

  void updateFilter(
    String newInputText,
    String newFilter, {
    int? newEndPosition,
  }) {
    updateInputText(newInputText);

    // filter the views
    _filter = newFilter.toLowerCase();
    if (newEndPosition != null) {
      _filterEndPosition = newEndPosition;
    }

    final newVisibleViews =
        allViews.where((v) => !selectedViewIds.contains(v.id)).toList();

    if (_filter.isNotEmpty) {
      newVisibleViews.retainWhere(
        (v) {
          final nonEmptyName = v.name.isEmpty
              ? LocaleKeys.document_title_placeholder.tr()
              : v.name;
          return nonEmptyName.toLowerCase().contains(_filter);
        },
      );
    }

    state.maybeWhen(
      ready: (_, oldFocusedIndex) {
        final newFocusedViewIndex = oldFocusedIndex < newVisibleViews.length
            ? oldFocusedIndex
            : (newVisibleViews.isEmpty ? -1 : 0);
        emit(
          ChatInputControlState.ready(
            visibleViews: newVisibleViews,
            focusedViewIndex: newFocusedViewIndex,
          ),
        );
      },
      orElse: () {},
    );
  }

  void updateInputText(String newInputText) {
    _inputText = newInputText;

    // input text is changed, see if there are any deletions
    selectedViewIds.retainWhere(_inputText.contains);
    _notifyUpdateSelectedViews();
  }

  void updateSelectionUp() {
    state.maybeMap(
      ready: (readyState) {
        final newIndex = readyState.visibleViews.isEmpty
            ? -1
            : (readyState.focusedViewIndex - 1) %
                readyState.visibleViews.length;
        emit(
          readyState.copyWith(focusedViewIndex: newIndex),
        );
      },
      orElse: () {},
    );
  }

  void updateSelectionDown() {
    state.maybeMap(
      ready: (readyState) {
        final newIndex = readyState.visibleViews.isEmpty
            ? -1
            : (readyState.focusedViewIndex + 1) %
                readyState.visibleViews.length;
        emit(
          readyState.copyWith(focusedViewIndex: newIndex),
        );
      },
      orElse: () {},
    );
  }

  void selectPage(ViewPB view) {
    selectedViewIds.add(view.id);
    _notifyUpdateSelectedViews();
    reset();
  }

  String formatIntputText(final String input) {
    String result = input;
    for (final viewId in selectedViewIds) {
      if (!result.contains(viewId)) {
        continue;
      }
      final view = allViews.firstWhereOrNull((view) => view.id == viewId);
      if (view != null) {
        final nonEmptyName = view.name.isEmpty
            ? LocaleKeys.document_title_placeholder.tr()
            : view.name;
        result = result.replaceAll(RegExp(viewId), nonEmptyName);
      }
    }
    return result;
  }

  void _notifyUpdateSelectedViews() {
    final stateCopy = state;
    final selectedViews =
        allViews.where((view) => selectedViewIds.contains(view.id)).toList();
    emit(ChatInputControlState.updateSelectedViews(selectedViews));
    emit(stateCopy);
  }

  /// Enable mention tracking by listening to text changes in the controller
  /// This method should be called when the text input field is ready to use
  void enableMentionTracking(TextEditingController controller) {
    _textController = controller;
    controller.addListener(_onTextChanged);
  }

  /// Disable mention tracking and remove the listener
  /// This method should be called when the controller is being disposed
  void disableMentionTracking() {
    if (_textController != null) {
      _textController!.removeListener(_onTextChanged);
      _textController = null;
    }
    _isAtMentionMode = false;
    _atSymbolPosition = -1;
  }

  /// Handle text changes and detect @ symbol for mention mode
  void _onTextChanged() {
    final controller = _textController;
    if (controller == null) return;

    final text = controller.text;
    final selection = controller.selection;
    final cursorPosition = selection.baseOffset;

    // Skip processing if text is empty
    if (text.isEmpty) {
      _cancelAtMention();
      return;
    }

    // Detect @ symbol at cursor position
    if (cursorPosition > 0 && cursorPosition <= text.length) {
      final charBeforeCursor = text[cursorPosition - 1];
      
      if (charBeforeCursor == '@') {
        // @ symbol detected, start mention mode
        _startAtMention(text, cursorPosition);
      } else if (_isAtMentionMode) {
        // Check if we're still in mention mode
        if (cursorPosition > _atSymbolPosition &&
            _atSymbolPosition >= 0 &&
            _atSymbolPosition < text.length) {
          // Cursor is after @ symbol, continue mention mode
          _continueAtMention(text, cursorPosition);
        } else {
          // Cursor moved before @ or @ symbol was deleted, cancel mention mode
          _cancelAtMention();
        }
      }
    } else {
      // Cursor at start or invalid position
      if (_isAtMentionMode) {
        _cancelAtMention();
      }
    }
  }

  /// Start @mention mode when @ symbol is detected
  void _startAtMention(String text, int cursorPosition) {
    _isAtMentionMode = true;
    _atSymbolPosition = cursorPosition - 1; // Position of @ symbol

    // Update filter start position to match @ symbol position
    _filterStartPosition = cursorPosition;
    _filterEndPosition = cursorPosition;
    _filter = "";
    _inputText = text;

    // Refresh views and emit state with mention mode active
    refreshViews();
  }

  /// Continue @mention mode when user types after @ symbol
  void _continueAtMention(String text, int cursorPosition) {
    if (!_isAtMentionMode || _atSymbolPosition < 0) {
      return;
    }

    // Extract filter text between @ and cursor
    if (cursorPosition > _atSymbolPosition + 1) {
      _filter = text.substring(_atSymbolPosition + 1, cursorPosition);
      _filterEndPosition = cursorPosition;
    } else {
      _filter = "";
      _filterEndPosition = _atSymbolPosition + 1;
    }

    _inputText = text;

    // Use existing updateFilter method to handle filtering
    updateFilter(text, _filter, newEndPosition: _filterEndPosition);
  }

  /// Cancel @mention mode
  void _cancelAtMention() {
    _isAtMentionMode = false;
    _atSymbolPosition = -1;
  }

  /// Get document ID by name from allViews
  /// Returns null if document not found
  String? getDocumentIdByName(String documentName) {
    if (documentName.isEmpty) {
      return null;
    }
    
    final view = allViews.firstWhereOrNull(
      (view) {
        final nonEmptyName = view.name.isEmpty
            ? LocaleKeys.document_title_placeholder.tr()
            : view.name;
        return nonEmptyName == documentName;
      },
    );
    
    return view?.id;
  }

  /// Get document ID mapping (name -> ViewPB)
  /// Returns a map where key is document name and value is ViewPB
  Map<String, ViewPB> getDocumentIdMap() {
    final Map<String, ViewPB> map = {};
    for (final view in allViews) {
      final nonEmptyName = view.name.isEmpty
          ? LocaleKeys.document_title_placeholder.tr()
          : view.name;
      // If multiple documents have the same name, keep the last one
      map[nonEmptyName] = view;
    }
    return map;
  }

  @override
  Future<void> close() {
    disableMentionTracking();
    return super.close();
  }
}

@freezed
class ChatInputControlState with _$ChatInputControlState {
  const factory ChatInputControlState.loading() = _Loading;

  const factory ChatInputControlState.ready({
    required List<ViewPB> visibleViews,
    required int focusedViewIndex,
  }) = _Ready;

  const factory ChatInputControlState.updateSelectedViews(
    List<ViewPB> selectedViews,
  ) = _UpdateOneShot;
}
