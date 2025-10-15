import 'dart:async';
import 'dart:typed_data';

import 'package:appflowy/plugins/ai_chat/application/chat_notification.dart';
import 'package:appflowy_backend/dispatch/dispatch.dart';
import 'package:appflowy_backend/log.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/notification.pb.dart';
import 'package:appflowy_backend/protobuf/flowy-error/errors.pb.dart';
import 'package:appflowy_backend/rust_stream.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'vector_index_bloc.freezed.dart';

class VectorIndexBloc extends Bloc<VectorIndexEvent, VectorIndexState> {
  VectorIndexBloc() : super(const VectorIndexState.initial()) {
    on<VectorIndexEvent>(_handleEvent);
    
    _parser = ChatNotificationParser(
      id: 'VectorIndexNotification',
      callback: _onNotification,
    );
    _subscription = RustStreamReceiver.listen(
      (observable) => _parser?.parse(observable),
    );
  }

  ChatNotificationParser? _parser;
  StreamSubscription? _subscription;

  Future<void> _handleEvent(
    VectorIndexEvent event,
    Emitter<VectorIndexState> emit,
  ) async {
    await event.when(
      started: () async {
        // 初始化时获取当前状态
        await _fetchStatus(emit);
      },
      rebuildIndex: () async {
        // 调用后端重建索引
        final result = await AIEventRebuildVectorIndex(
          RebuildVectorIndexRequestPB(documentIds: []),
        ).send();

        await result.fold(
          (response) async {
            if (!response.success && response.hasError()) {
              Log.error('重建索引失败: ${response.error}');
            }
            // 无论成功与否，都获取最新状态
            await _fetchStatus(emit);
          },
          (error) async {
            Log.error('重建索引请求失败: $error');
            await _fetchStatus(emit);
          },
        );
      },
      stopIndexing: () async {
        // 调用后端停止索引
        final result = await AIEventStopVectorIndexing().send();

        await result.fold(
          (_) async {
            await _fetchStatus(emit);
          },
          (error) async {
            Log.error('停止索引失败: $error');
            await _fetchStatus(emit);
          },
        );
      },
      statusUpdated: (status) {
        // 收到通知更新，更新状态
        emit(state.copyWith(
          status: status,
          isRunning: status.state == VectorIndexStatePB.IndexRunning,
        ));
      },
    );
  }

  Future<void> _fetchStatus(Emitter<VectorIndexState> emit) async {
    final result = await AIEventGetVectorIndexStatus().send();

    result.fold(
      (status) {
        emit(state.copyWith(
          status: status,
          isRunning: status.state == VectorIndexStatePB.IndexRunning,
        ));
      },
      (error) {
        Log.error('获取索引状态失败: $error');
      },
    );
  }

  void _onNotification(
    ChatNotification ty,
    FlowyResult<Uint8List, FlowyError> result,
  ) {
    result.map((payload) {
      if (ty == ChatNotification.VectorIndexStatusUpdated) {
        try {
          final status = VectorIndexStatusPB.fromBuffer(payload);
          if (!isClosed) {
            add(VectorIndexEvent.statusUpdated(status));
          }
        } catch (e) {
          Log.error('解析向量索引状态通知失败: $e');
        }
      }
    });
  }

  @override
  Future<void> close() {
    _parser = null;
    _subscription?.cancel();
    return super.close();
  }
}

@freezed
class VectorIndexEvent with _$VectorIndexEvent {
  const factory VectorIndexEvent.started() = _Started;
  const factory VectorIndexEvent.rebuildIndex() = _RebuildIndex;
  const factory VectorIndexEvent.stopIndexing() = _StopIndexing;
  const factory VectorIndexEvent.statusUpdated(VectorIndexStatusPB status) = _StatusUpdated;
}

@freezed
class VectorIndexState with _$VectorIndexState {
  const factory VectorIndexState.initial({
    @Default(null) VectorIndexStatusPB? status,
    @Default(false) bool isRunning,
  }) = _Initial;

  factory VectorIndexState.fromStatus(VectorIndexStatusPB status) {
    return VectorIndexState.initial(
      status: status,
      isRunning: status.state == VectorIndexStatePB.IndexRunning,
    );
  }
}

